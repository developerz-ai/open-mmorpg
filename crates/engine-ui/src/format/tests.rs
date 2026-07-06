//! Unit tests for locale-aware number, currency, percent, and date formatting.

use super::*;

#[test]
fn integer_grouping_per_locale() {
    let en = LocaleFormatter::new("en");
    let de = LocaleFormatter::new("de");
    let fr = LocaleFormatter::new("fr");
    assert_eq!(en.format_integer(1_234_567), "1,234,567");
    assert_eq!(de.format_integer(1_234_567), "1.234.567");
    assert_eq!(fr.format_integer(1_234_567), "1 234 567");
    assert_eq!(en.format_integer(-42), "-42");
    assert_eq!(en.format_integer(0), "0");
    assert_eq!(en.format_integer(999), "999");
    assert_eq!(en.format_integer(1_000), "1,000");
}

#[test]
fn integer_handles_i64_min_without_overflow() {
    let en = LocaleFormatter::new("en");
    assert_eq!(en.format_integer(i64::MIN), "-9,223,372,036,854,775,808");
}

#[test]
fn decimal_uses_locale_separators_and_rounds() {
    let en = LocaleFormatter::new("en");
    let de = LocaleFormatter::new("de");
    assert_eq!(en.format_decimal(1234.5, 2), "1,234.50");
    assert_eq!(de.format_decimal(1234.5, 2), "1.234,50");
    assert_eq!(en.format_decimal(2.0 / 3.0, 3), "0.667");
    // No spurious negative zero.
    assert_eq!(en.format_decimal(-0.001, 2), "0.00");
    assert_eq!(en.format_decimal(-1.5, 1), "-1.5");
}

#[test]
fn currency_placement_and_minor_units() {
    let en = LocaleFormatter::new("en");
    let de = LocaleFormatter::new("de");
    let ja = LocaleFormatter::new("ja");
    assert_eq!(en.format_currency(1_234_567, Currency::USD), "$12,345.67");
    assert_eq!(de.format_currency(1_234_567, Currency::EUR), "12.345,67 €");
    // JPY has zero minor digits — no decimal part.
    assert_eq!(ja.format_currency(1_234_567, Currency::JPY), "¥1,234,567");
    assert_eq!(en.format_currency(-500, Currency::USD), "-$5.00");
    assert_eq!(en.format_currency(5, Currency::USD), "$0.05");
}

#[test]
fn percent_spacing_per_locale() {
    let en = LocaleFormatter::new("en");
    let fr = LocaleFormatter::new("fr");
    assert_eq!(en.format_percent(0.5, 0), "50%");
    assert_eq!(en.format_percent(0.1234, 1), "12.3%");
    assert_eq!(fr.format_percent(0.5, 0), "50 %");
}

#[test]
fn date_order_and_separator_per_locale() {
    let us = LocaleFormatter::new("en-US");
    let gb = LocaleFormatter::new("en-GB");
    let de = LocaleFormatter::new("de");
    let ja = LocaleFormatter::new("ja");
    assert_eq!(us.format_date(2026, 7, 6), "07/06/2026");
    assert_eq!(gb.format_date(2026, 7, 6), "06/07/2026");
    assert_eq!(de.format_date(2026, 7, 6), "06.07.2026");
    assert_eq!(ja.format_date(2026, 7, 6), "2026/07/06");
}

#[test]
fn unknown_locale_falls_back_to_en_rules() {
    let xx = LocaleFormatter::new("xx-YY");
    assert_eq!(xx.locale(), "xx-YY");
    assert_eq!(xx.format_integer(1_000), "1,000");
    assert_eq!(xx.format_date(2026, 7, 6), "07/06/2026");
}

#[test]
fn underscore_locale_separator_is_accepted() {
    let de = LocaleFormatter::new("de_DE");
    assert_eq!(de.format_integer(1_000), "1.000");
}
