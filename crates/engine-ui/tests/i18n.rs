//! End-to-end i18n substrate test: the public surface a HUD would drive.
//! Strings resolve through `t(key, args)` with loud missing markers; plurals and
//! gender come from Fluent; numbers, money, and dates are formatted outside the
//! catalog via [`LocaleFormatter`].

use omm_engine_ui::{Currency, I18nBundle, LocaleFormatter, TransArgs};

const EN: &str = r#"
hud-gold = Gold
quest-progress = { $done } of { $total } complete
enemies-left = { $n ->
    [one] { $n } enemy remaining
   *[other] { $n } enemies remaining
}
"#;

const DE: &str = r#"
hud-gold = Gold
enemies-left = { $n ->
    [one] Noch { $n } Gegner
   *[other] Noch { $n } Gegner
}
"#;

#[test]
fn hud_strings_resolve_with_plurals_and_fallback() {
    let mut ui = I18nBundle::from_sources("en", [("en", EN), ("de", DE)]).expect("valid ftl");

    assert_eq!(ui.t("hud-gold", &TransArgs::new()), "Gold");
    assert_eq!(
        ui.t("enemies-left", &TransArgs::new().set("n", 1_i64)),
        "1 enemy remaining"
    );
    assert_eq!(
        ui.t("enemies-left", &TransArgs::new().set("n", 7_i64)),
        "7 enemies remaining"
    );

    // Switch locale: de resolves its own strings, falls back to en for missing.
    ui.set_locale("de");
    assert_eq!(ui.t("hud-gold", &TransArgs::new()), "Gold");
    assert_eq!(
        ui.t("enemies-left", &TransArgs::new().set("n", 3_i64)),
        "Noch 3 Gegner"
    );
    // `quest-progress` only exists in en → falls back, never blank.
    let progress = ui.t(
        "quest-progress",
        &TransArgs::new().set("done", 2_i64).set("total", 5_i64),
    );
    assert_eq!(progress, "2 of 5 complete");
}

#[test]
fn missing_keys_render_loudly() {
    let ui = I18nBundle::from_sources("en", [("en", EN)]).expect("valid ftl");
    assert_eq!(
        ui.t("does-not-exist", &TransArgs::new()),
        "⟦does-not-exist⟧"
    );
}

#[test]
fn numbers_money_and_dates_come_from_formatter_not_catalog() {
    // These are never keys in the catalog — they are formatted by locale rules.
    let en = LocaleFormatter::new("en-US");
    let de = LocaleFormatter::new("de");

    assert_eq!(en.format_integer(2_500_000), "2,500,000");
    assert_eq!(de.format_integer(2_500_000), "2.500.000");

    assert_eq!(en.format_currency(1_999, Currency::USD), "$19.99");
    assert_eq!(de.format_currency(1_999, Currency::EUR), "19,99 €");

    assert_eq!(en.format_date(2026, 7, 6), "07/06/2026");
    assert_eq!(de.format_date(2026, 7, 6), "06.07.2026");

    assert_eq!(en.format_percent(0.73, 0), "73%");
}
