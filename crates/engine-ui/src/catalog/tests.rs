//! Unit tests for the pure i18n catalog: lookup, interpolation, plurals,
//! gender/select, attributes, fallback, and loud missing keys.

use super::*;

const EN: &str = r#"
hello = Hello!
greet = Hello, { $name }!
items = { $count ->
    [one] { $count } item
   *[other] { $count } items
}
welcome = { $gender ->
    [male] Welcome, sir
    [female] Welcome, madam
   *[other] Welcome
}
login =
    .submit = Sign in
    .cancel = Cancel
"#;

const FR: &str = r#"
hello = Bonjour !
greet = Bonjour, { $name } !
"#;

// Polish exercises CLDR plural categories beyond one/other: one/few/many.
const PL: &str = r#"
files = { $n ->
    [one] { $n } plik
    [few] { $n } pliki
   *[many] { $n } plików
}
"#;

fn en() -> Catalog {
    Catalog::from_sources([("en", EN)]).expect("valid ftl")
}

#[test]
fn split_key_separates_message_and_attribute() {
    assert_eq!(split_key("hello"), ("hello", None));
    assert_eq!(split_key("login.submit"), ("login", Some("submit")));
}

#[test]
fn resolves_plain_message() {
    let c = en();
    assert_eq!(c.t("en", "hello", &TransArgs::new()), "Hello!");
}

#[test]
fn interpolates_named_argument() {
    let c = en();
    let args = TransArgs::new().set("name", "Ada");
    assert_eq!(c.t("en", "greet", &args), "Hello, Ada!");
}

#[test]
fn missing_key_renders_loud_marker() {
    let c = en();
    assert_eq!(c.t("en", "nope", &TransArgs::new()), "⟦nope⟧");
    assert_eq!(missing_marker("a.b"), "⟦a.b⟧");
}

#[test]
fn plural_selection_english_one_other() {
    let c = en();
    let one = c.t("en", "items", &TransArgs::new().set("count", 1_i64));
    let many = c.t("en", "items", &TransArgs::new().set("count", 5_i64));
    assert_eq!(one, "1 item");
    assert_eq!(many, "5 items");
}

#[test]
fn plural_selection_polish_one_few_many() {
    let c = Catalog::from_sources([("pl", PL)]).expect("valid ftl");
    assert_eq!(
        c.t("pl", "files", &TransArgs::new().set("n", 1_i64)),
        "1 plik"
    );
    assert_eq!(
        c.t("pl", "files", &TransArgs::new().set("n", 3_i64)),
        "3 pliki"
    );
    assert_eq!(
        c.t("pl", "files", &TransArgs::new().set("n", 5_i64)),
        "5 plików"
    );
}

#[test]
fn gender_select_branches() {
    let c = en();
    let male = c.t("en", "welcome", &TransArgs::new().set("gender", "male"));
    let female = c.t("en", "welcome", &TransArgs::new().set("gender", "female"));
    let other = c.t("en", "welcome", &TransArgs::new().set("gender", "nb"));
    assert_eq!(male, "Welcome, sir");
    assert_eq!(female, "Welcome, madam");
    assert_eq!(other, "Welcome");
}

#[test]
fn attribute_lookup_via_dotted_key() {
    let c = en();
    assert_eq!(c.t("en", "login.submit", &TransArgs::new()), "Sign in");
    assert_eq!(c.t("en", "login.cancel", &TransArgs::new()), "Cancel");
    // Unknown attribute → loud marker, not a silent blank.
    assert_eq!(
        c.t("en", "login.unknown", &TransArgs::new()),
        "⟦login.unknown⟧"
    );
}

#[test]
fn fallback_chain_truncates_then_defaults() {
    let c = Catalog::from_sources([("en", EN), ("fr", FR)]).expect("valid ftl");
    // fr-CA has no bundle → falls back to fr for a key fr defines.
    assert_eq!(c.t("fr-CA", "hello", &TransArgs::new()), "Bonjour !");
    // fr lacks `items` → falls back to the default locale (en).
    let en_plural = c.t("fr-CA", "items", &TransArgs::new().set("count", 2_i64));
    assert_eq!(en_plural, "2 items");
}

#[test]
fn fallback_chain_order_is_specific_to_general() {
    let c = Catalog::from_sources([("en", EN)]).expect("valid ftl");
    assert_eq!(c.fallback_chain("fr-CA"), vec!["fr-CA", "fr", "en"]);
    assert_eq!(c.fallback_chain("en"), vec!["en"]);
}

#[test]
fn supports_reflects_prefix_not_default() {
    let c = Catalog::from_sources([("en", EN), ("fr", FR)]).expect("valid ftl");
    assert!(c.has_locale("fr"));
    assert!(!c.has_locale("fr-CA"));
    assert!(c.supports("fr-CA")); // fr prefix exists
    assert!(!c.supports("de")); // only default fallback would apply
    assert_eq!(c.default_locale(), "en");
}

#[test]
fn malformed_source_is_a_typed_error() {
    let err = Catalog::from_sources([("en", "greet = { $x")]);
    assert!(matches!(err, Err(I18nError::FluentParse { .. })));
}

#[test]
fn deterministic_same_inputs_same_output() {
    let c = en();
    let a = c.t("en", "greet", &TransArgs::new().set("name", "Zoe"));
    let b = c.t("en", "greet", &TransArgs::new().set("name", "Zoe"));
    assert_eq!(a, b);
}
