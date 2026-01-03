use crate::locale::Locale;
use crate::num_formats::ToFormattedString;

#[test]
fn test_u8_primitive() {
    assert_eq!(255u8.to_formatted_string(&Locale::en), "255");
}

#[test]
fn test_i8_negative() {
    assert_eq!((-128i8).to_formatted_string(&Locale::en), "-128");
}

#[test]
fn test_u128_large() {
    let val: u128 = 1_000_000_000_000_000_000;
    assert_eq!(
        val.to_formatted_string(&Locale::en),
        "1,000,000,000,000,000,000"
    );
}

#[test]
fn test_isize_boundaries() {
    assert_eq!(0isize.to_formatted_string(&Locale::en), "0");
}

#[test]
fn test_f32_standard() {
    let val: f32 = 1234.5;
    // format!() for f32 1234.5 usually gives "1234.5"
    assert_eq!(val.to_formatted_string(&Locale::en), "1,234.5");
}

#[test]
fn test_f64_german_separator() {
    // German uses ',' for decimal and '.' for grouping
    let val: f64 = 1234.56;
    assert_eq!(val.to_formatted_string(&Locale::de), "1.234,56");
}

#[test]
fn test_f64_negative() {
    let val: f64 = -10.5;
    assert_eq!(val.to_formatted_string(&Locale::en), "-10.5");
}

#[test]
fn test_grouping_en_us() {
    // Standard 3-digit grouping
    assert_eq!(1000000.to_formatted_string(&Locale::en), "1,000,000");
}

#[test]
fn test_grouping_hi_in() {
    // Indian system: 1,00,00,000 (3-2-2)
    let val = 10000000;
    assert_eq!(val.to_formatted_string(&Locale::hi), "1,00,00,000");
}

#[test]
fn test_grouping_boundary_exact() {
    // Test exactly at the first grouping boundary
    assert_eq!(100.to_formatted_string(&Locale::en), "100");
    assert_eq!(1000.to_formatted_string(&Locale::en), "1,000");
}

#[test]
fn test_separator_french_space() {
    let res = 1000.to_formatted_string(&Locale::fr);
    // CLDR often uses U+00A0 (Non-breaking space) or U+202F (Narrow NBSP)
    let has_space = res
        .chars()
        .any(|c| c.is_whitespace() || c == '\u{a0}' || c == '\u{202f}');
    assert!(has_space, "French result '{}' should contain a space", res);
}

#[test]
fn test_separator_swiss_apostrophe() {
    // Some Swiss variations use '
    // Note: Depends on your specific CLDR version/variant
    let res = 1000.to_formatted_string(&Locale::gsw);
    assert!(res.contains('\'') || res.contains(' '));
}

#[test]
fn test_sign_is_not_grouped() {
    // Ensure the minus sign doesn't accidentally get treated as a digit
    assert_eq!((-10000).to_formatted_string(&Locale::en), "-10,000");
}

#[test]
fn test_zero_handling() {
    assert_eq!(0.to_formatted_string(&Locale::en), "0");
    assert_eq!(0.0f64.to_formatted_string(&Locale::en), "0");
}

#[test]
fn arabic_numbers() {
    // ar_EG (Egypt) or ar_SA (Saudi Arabia) typically default to 'arab' digits
    assert_eq!(0.to_formatted_string(&Locale::ar_EG), "٠");
    assert_eq!(123.to_formatted_string(&Locale::ar_EG), "١٢٣");
    assert_eq!(
        0123456789.to_formatted_string(&Locale::ar_EG),
        "١٢٣٬٤٥٦٬٧٨٩"
    )
}
