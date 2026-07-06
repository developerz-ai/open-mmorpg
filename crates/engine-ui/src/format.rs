//! Locale-aware formatting for numbers, currency, percent, and dates.
//!
//! This is the `Intl`-equivalent surface: values are formatted **programmatically
//! from locale rules**, never stored as strings in the i18n catalog (the rule from
//! the [UI spec] and `apps/web`). Money is carried as **integer minor units**
//! (e.g. cents) to avoid floating-point drift.
//!
//! Coverage is a curated locale set (the launch languages); unknown locales fall
//! back to `en` rules. Group separators use an ASCII space where CLDR prefers a
//! narrow no-break space — an honest, documented simplification with a clean path
//! to swapping in full ICU4X data behind this same API.
//!
//! [UI spec]: ../../../docs/specs/game-engine/ui/README.md

/// Grouping / decimal separators for a locale's number system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NumberSpec {
    decimal: char,
    group: char,
    group_size: usize,
}

/// Order of numeric date components for a locale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DateOrder {
    /// Year-Month-Day (e.g. `ja`, `zh`).
    Ymd,
    /// Day-Month-Year (most of Europe).
    Dmy,
    /// Month-Day-Year (`en-US`).
    Mdy,
}

/// A currency: ISO 4217 code, display symbol, and minor-unit digit count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Currency {
    /// ISO 4217 code, e.g. `"USD"`.
    pub code: &'static str,
    /// Display symbol, e.g. `"$"`.
    pub symbol: &'static str,
    /// Number of minor-unit digits (2 for USD/EUR, 0 for JPY).
    pub minor_digits: u32,
}

impl Currency {
    /// US dollar.
    pub const USD: Currency = Currency {
        code: "USD",
        symbol: "$",
        minor_digits: 2,
    };
    /// Euro.
    pub const EUR: Currency = Currency {
        code: "EUR",
        symbol: "€",
        minor_digits: 2,
    };
    /// Pound sterling.
    pub const GBP: Currency = Currency {
        code: "GBP",
        symbol: "£",
        minor_digits: 2,
    };
    /// Japanese yen (no minor units).
    pub const JPY: Currency = Currency {
        code: "JPY",
        symbol: "¥",
        minor_digits: 0,
    };
}

/// Formats numbers, money, and dates for a single locale.
#[derive(Clone, Debug)]
pub struct LocaleFormatter {
    locale: String,
    number: NumberSpec,
    date: DateOrder,
    date_sep: char,
    /// Currency symbol placed before the amount (`true`) or after (`false`);
    /// also drives whether percents take a leading space.
    symbol_prefix: bool,
}

impl Default for LocaleFormatter {
    fn default() -> Self {
        Self::new("en")
    }
}

impl LocaleFormatter {
    /// Build a formatter for `locale` (language + optional region).
    #[must_use]
    pub fn new(locale: &str) -> Self {
        let mut parts = locale.split(['-', '_']);
        let lang = parts.next().unwrap_or("en");
        let region = parts.next();
        let (number, date, date_sep, symbol_prefix) = rules_for(lang, region);
        Self {
            locale: locale.to_owned(),
            number,
            date,
            date_sep,
            symbol_prefix,
        }
    }

    /// The locale this formatter was built for.
    #[must_use]
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Insert grouping separators into a run of ASCII decimal digits.
    fn group(&self, digits: &str) -> String {
        let n = digits.len();
        if n <= self.number.group_size {
            return digits.to_owned();
        }
        let size = self.number.group_size;
        let mut out = String::with_capacity(n + n / size);
        let head = match n % size {
            0 => size,
            r => r,
        };
        out.push_str(&digits[..head]);
        let mut i = head;
        while i < n {
            out.push(self.number.group);
            out.push_str(&digits[i..i + size]);
            i += size;
        }
        out
    }

    /// Format an integer with locale grouping, e.g. `1234567` → `1,234,567`.
    #[must_use]
    pub fn format_integer(&self, value: i64) -> String {
        let grouped = self.group(&value.unsigned_abs().to_string());
        if value < 0 {
            format!("-{grouped}")
        } else {
            grouped
        }
    }

    /// Format a float with `frac_digits` fractional digits and locale separators.
    #[must_use]
    pub fn format_decimal(&self, value: f64, frac_digits: usize) -> String {
        let mag = value.abs();
        let s = format!("{mag:.frac_digits$}");
        let (int_part, frac_part) = s.split_once('.').unwrap_or((s.as_str(), ""));
        // Avoid a spurious "-0.00": only negative if a non-zero digit survives.
        let neg = value.is_sign_negative() && s.bytes().any(|b| b.is_ascii_digit() && b != b'0');
        let mut out = String::new();
        if neg {
            out.push('-');
        }
        out.push_str(&self.group(int_part));
        if !frac_part.is_empty() {
            out.push(self.number.decimal);
            out.push_str(frac_part);
        }
        out
    }

    /// Format a ratio (`0.5` → `50%`) with locale-appropriate spacing.
    #[must_use]
    pub fn format_percent(&self, ratio: f64, frac_digits: usize) -> String {
        let pct = self.format_decimal(ratio * 100.0, frac_digits);
        if self.symbol_prefix {
            format!("{pct}%")
        } else {
            format!("{pct} %")
        }
    }

    /// Format money given as integer minor units, e.g. `1234567` USD → `$12,345.67`.
    #[must_use]
    pub fn format_currency(&self, minor_units: i64, currency: Currency) -> String {
        let mag = minor_units.unsigned_abs();
        let divisor = 10u64.pow(currency.minor_digits);
        let mut amount = self.group(&(mag / divisor).to_string());
        if currency.minor_digits > 0 {
            let frac = mag % divisor;
            let width = currency.minor_digits as usize;
            amount.push(self.number.decimal);
            amount.push_str(&format!("{frac:0width$}"));
        }
        let body = if self.symbol_prefix {
            format!("{}{amount}", currency.symbol)
        } else {
            format!("{amount} {}", currency.symbol)
        };
        if minor_units < 0 {
            format!("-{body}")
        } else {
            body
        }
    }

    /// Format a numeric date in the locale's component order and separator.
    #[must_use]
    pub fn format_date(&self, year: i32, month: u32, day: u32) -> String {
        let sep = self.date_sep;
        match self.date {
            DateOrder::Ymd => format!("{year:04}{sep}{month:02}{sep}{day:02}"),
            DateOrder::Dmy => format!("{day:02}{sep}{month:02}{sep}{year:04}"),
            DateOrder::Mdy => format!("{month:02}{sep}{day:02}{sep}{year:04}"),
        }
    }
}

/// Locale rules table for the curated launch set. Returns
/// `(number spec, date order, date separator, currency-symbol-prefix)`.
fn rules_for(lang: &str, region: Option<&str>) -> (NumberSpec, DateOrder, char, bool) {
    let dot_comma = NumberSpec {
        decimal: '.',
        group: ',',
        group_size: 3,
    };
    let comma_dot = NumberSpec {
        decimal: ',',
        group: '.',
        group_size: 3,
    };
    let comma_space = NumberSpec {
        decimal: ',',
        group: ' ',
        group_size: 3,
    };
    match lang {
        "en" => match region {
            Some("GB") => (dot_comma, DateOrder::Dmy, '/', true),
            _ => (dot_comma, DateOrder::Mdy, '/', true),
        },
        "de" => (comma_dot, DateOrder::Dmy, '.', false),
        "fr" => (comma_space, DateOrder::Dmy, '/', false),
        "es" | "it" | "pt" => (comma_dot, DateOrder::Dmy, '/', false),
        "pl" | "ru" => (comma_space, DateOrder::Dmy, '.', false),
        "ja" | "zh" => (dot_comma, DateOrder::Ymd, '/', true),
        _ => (dot_comma, DateOrder::Mdy, '/', true),
    }
}

#[cfg(test)]
mod tests;
