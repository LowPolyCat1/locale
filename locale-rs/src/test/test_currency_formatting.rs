use crate::currency_formats::ToCurrencyString;
use crate::locale::Locale;
use std::str::FromStr;

#[test]
fn test_currency_placement_and_symbol() {
    // Test English (Typically Prefix: $1.99)
    let locale = Locale::from_str("en").unwrap();
    let en_res = 1.99.to_currency(&locale);
    assert!(
        en_res.starts_with('$'),
        "English currency should start with symbol: {}",
        en_res
    );
    assert_eq!(en_res, "$1.99");

    // Test German (Typically Suffix: 1,99 €)
    // Note: CLDR uses a non-breaking space (U+00A0) between number and symbol
    let locale = Locale::from_str("de").unwrap();
    let de_res = 1.99.to_currency(&locale);
    assert!(
        de_res.contains("€"),
        "German currency should contain symbol: {}",
        de_res
    );
    assert!(
        de_res.contains("1,99"),
        "German should use comma decimal: {}",
        de_res
    );
}

#[test]
fn test_whole_number_dash_logic() {
    let locale = Locale::from_str("en").unwrap();
    let locale1 = Locale::from_str("de").unwrap();

    // 100.0 -> "100,-"
    assert_eq!(100.0.to_currency(&locale), "$100,-");

    // 1000.0 -> "1.000,-" (German grouping)
    let de_res = 1000.0.to_currency(&locale1);
    assert!(
        de_res.contains("1.000,-"),
        "German grouping failed: {}",
        de_res
    );
}

#[test]
fn test_negative_formatting() {
    let locale = Locale::from_str("en").unwrap();
    let locale2 = Locale::from_str("de").unwrap();

    // Negative decimal
    assert_eq!((-1.99).to_currency(&locale), "-$1.99");

    // Negative whole
    assert_eq!((-50.0).to_currency(&locale2), "-50,-\u{a0}€");

    // Negative German
    let de_res = (-50.0).to_currency(&locale2);
    assert!(de_res.starts_with('-'), "Negative sign missing: {}", de_res);
}

#[test]
fn test_large_numbers_and_grouping() {
    let locale = Locale::from_str("en").unwrap();
    let locale2 = Locale::from_str("de").unwrap();

    // Millions with grouping
    assert_eq!(1234567.89.to_currency(&locale), "$1,234,567.89");

    let de_res = 1234567.89.to_currency(&locale2);
    assert!(
        de_res.contains("1.234.567,89"),
        "German large number grouping failed: {}",
        de_res
    );
}

#[test]
fn test_rounding_edge_cases() {
    let locale = Locale::from_str("en").unwrap();

    // Rounding up
    assert_eq!(1.999.to_currency(&locale), "$2,-"); // Since 2.00 is whole

    // Rounding down
    assert_eq!(1.994.to_currency(&locale), "$1.99");
}

#[test]
fn test_all_primitive_types() {
    let loc = Locale::from_str("en").unwrap();

    let val_i32: i32 = 42;
    let val_u64: u64 = 5000;
    let val_f32: f32 = 12.34;
    let val_isize: isize = -10;

    assert_eq!(val_i32.to_currency(&loc), "$42,-");
    assert_eq!(val_u64.to_currency(&loc), "$5,000,-");
    assert_eq!(val_f32.to_currency(&loc), "$12.34");
    assert_eq!(val_isize.to_currency(&loc), "-$10,-");
}

#[test]
fn test_zero_values() {
    let locale = Locale::from_str("en").unwrap();
    assert_eq!(0.0.to_currency(&locale), "$0,-");
    assert_eq!(0.to_currency(&locale), "$0,-");
}
