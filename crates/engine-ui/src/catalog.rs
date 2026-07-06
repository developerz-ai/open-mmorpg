//! Pure, headless i18n catalog: compiled Fluent bundles + `t(key, args)`.
//!
//! Missing keys render **loudly** as `⟦key⟧` — never silently blank — matching
//! the product-wide rule ([`CLAUDE.md`], `apps/web`). ICU plurals / gender /
//! select come from Fluent selectors (`$n -> [one] … *[other] …`); dates & money
//! are formatted by [`crate::LocaleFormatter`], never stored in the catalog.
//!
//! [`CLAUDE.md`]: ../../../CLAUDE.md

use std::collections::HashMap;

use fluent::concurrent::FluentBundle;
use fluent::{FluentArgs, FluentResource, FluentValue};
use tracing::warn;
use unic_langid::LanguageIdentifier;

use crate::error::I18nError;

/// Split a lookup key into `(message_id, optional_attribute)`.
///
/// Fluent message ids cannot contain `.`; a dotted key like `login.submit`
/// addresses the `submit` attribute of the `login` message.
fn split_key(key: &str) -> (&str, Option<&str>) {
    match key.split_once('.') {
        Some((id, attr)) => (id, Some(attr)),
        None => (key, None),
    }
}

/// The loud marker rendered when a key resolves in no fallback locale.
#[must_use]
pub fn missing_marker(key: &str) -> String {
    format!("⟦{key}⟧")
}

/// Ergonomic builder for Fluent arguments.
///
/// Pass **numbers** to drive plural selection (`$n -> [one] … *[other] …`) and
/// **strings** to drive gender/select (`$g -> [male] … [female] … *[other] …`).
#[derive(Default)]
pub struct TransArgs<'a> {
    inner: FluentArgs<'a>,
}

impl<'a> TransArgs<'a> {
    /// An empty argument set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: FluentArgs::new(),
        }
    }

    /// Set an argument by name; chainable.
    #[must_use]
    pub fn set(mut self, key: &'a str, value: impl Into<FluentValue<'a>>) -> Self {
        self.inner.set(key, value);
        self
    }

    fn as_fluent(&self) -> &FluentArgs<'a> {
        &self.inner
    }
}

/// A set of compiled Fluent bundles keyed by BCP-47 locale, with a fallback chain.
///
/// Uses Fluent's concurrent memoizer, so it is `Send + Sync` and can live inside a
/// Bevy [`Resource`](bevy_ecs::system::Resource) shared across systems. Purely
/// in-memory: build it from FTL sources, query it with [`Catalog::t`]. No I/O.
pub struct Catalog {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    default_locale: String,
    use_isolating: bool,
}

impl Default for Catalog {
    fn default() -> Self {
        Self {
            bundles: HashMap::new(),
            default_locale: "en".to_owned(),
            use_isolating: false,
        }
    }
}

impl Catalog {
    /// Empty catalog with an explicit default (final-fallback) locale.
    #[must_use]
    pub fn new(default_locale: impl Into<String>) -> Self {
        Self {
            default_locale: default_locale.into(),
            ..Self::default()
        }
    }

    /// Toggle Unicode bidi isolation of interpolated args (FSI/PDI marks).
    ///
    /// Off by default for predictable, comparable output; enable for correct
    /// mixing of RTL and LTR runs in production UI.
    #[must_use]
    pub fn isolating(mut self, on: bool) -> Self {
        self.use_isolating = on;
        self
    }

    /// Build a catalog from `(locale, ftl_source)` pairs. The first pair's locale
    /// becomes the default (final-fallback) locale.
    pub fn from_sources<'s>(
        sources: impl IntoIterator<Item = (&'s str, &'s str)>,
    ) -> Result<Self, I18nError> {
        let mut catalog: Option<Self> = None;
        for (locale, source) in sources {
            let cat = catalog.get_or_insert_with(|| Self::new(locale));
            cat.add(locale, source)?;
        }
        Ok(catalog.unwrap_or_default())
    }

    /// Compile `source` and register (or replace) the bundle for `locale`.
    pub fn add(&mut self, locale: &str, source: &str) -> Result<(), I18nError> {
        let langid: LanguageIdentifier = locale.parse().unwrap_or_default();
        let mut bundle = FluentBundle::new_concurrent(vec![langid]);
        bundle.set_use_isolating(self.use_isolating);
        let resource = FluentResource::try_new(source.to_owned()).map_err(|(_, errs)| {
            I18nError::FluentParse {
                locale: locale.to_owned(),
                detail: format!("{errs:?}"),
            }
        })?;
        bundle
            .add_resource(resource)
            .map_err(|errs| I18nError::ResourceAdd {
                locale: locale.to_owned(),
                detail: format!("{errs:?}"),
            })?;
        self.bundles.insert(locale.to_owned(), bundle);
        Ok(())
    }

    /// Translate `key` for `locale`, walking the fallback chain.
    ///
    /// Returns `⟦key⟧` **loudly** (and warns) if the key resolves in no locale
    /// along the chain, so a missing string is impossible to miss on screen.
    pub fn t(&self, locale: &str, key: &str, args: &TransArgs) -> String {
        let (id, attr) = split_key(key);
        for loc in self.fallback_chain(locale) {
            let Some(bundle) = self.bundles.get(&loc) else {
                continue;
            };
            let Some(message) = bundle.get_message(id) else {
                continue;
            };
            let pattern = match attr {
                Some(a) => match message.get_attribute(a) {
                    Some(attribute) => attribute.value(),
                    None => continue,
                },
                None => match message.value() {
                    Some(value) => value,
                    None => continue,
                },
            };
            let mut errors = Vec::new();
            let out = bundle.format_pattern(pattern, Some(args.as_fluent()), &mut errors);
            if !errors.is_empty() {
                warn!(key, locale = %loc, ?errors, "i18n: format errors while resolving key");
            }
            return out.into_owned();
        }
        warn!(key, locale, "i18n: missing key — rendering loud marker");
        missing_marker(key)
    }

    /// Ordered locale keys to try: the requested locale, its progressively
    /// truncated parents (`fr-CA` → `fr`), then the default locale.
    fn fallback_chain(&self, locale: &str) -> Vec<String> {
        let mut chain = Vec::new();
        let mut cur = locale;
        loop {
            chain.push(cur.to_owned());
            match cur.rfind('-') {
                Some(idx) => cur = &cur[..idx],
                None => break,
            }
        }
        if !chain.iter().any(|l| l == &self.default_locale) {
            chain.push(self.default_locale.clone());
        }
        chain
    }

    /// True if a bundle is registered for this exact locale key.
    #[must_use]
    pub fn has_locale(&self, locale: &str) -> bool {
        self.bundles.contains_key(locale)
    }

    /// True if this locale or one of its parents has a bundle (ignores the
    /// default-locale fallback, so it reflects genuine support).
    #[must_use]
    pub fn supports(&self, locale: &str) -> bool {
        let mut cur = locale;
        loop {
            if self.bundles.contains_key(cur) {
                return true;
            }
            match cur.rfind('-') {
                Some(idx) => cur = &cur[..idx],
                None => return false,
            }
        }
    }

    /// The default (final-fallback) locale.
    #[must_use]
    pub fn default_locale(&self) -> &str {
        &self.default_locale
    }

    /// Registered locale keys (unordered).
    pub fn locales(&self) -> impl Iterator<Item = &str> {
        self.bundles.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests;
