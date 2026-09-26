use crate::locale::Locale;
use crate::nums::{NumberFormatter, NumberSymbols, ToFormattedString};

#[test]
fn test_diverse_numerical_symbols() {
    // Decimal separators.
    assert_eq!(1.23f64.to_formatted_string(&Locale::de), "1,23");
    assert_eq!(1.23f64.to_formatted_string(&Locale::en), "1.23");

    // Grouping separators.
    assert_eq!(1000.to_formatted_string(&Locale::de), "1.000");
    assert_eq!(1000.to_formatted_string(&Locale::de_CH), "1'000");

    // Locale-specific minus signs, e.g. U+2212 in Swedish.
    assert_eq!((-100).to_formatted_string(&Locale::sv), "\u{2212}100");
    let ar = NumberSymbols::for_locale(Locale::ar_EG);
    assert!(
        (-100)
            .to_formatted_string(&Locale::ar_EG)
            .starts_with(ar.minus_sign)
    );
}

#[test]
fn test_native_numbering_systems() {
    // ar-EG uses the `arab` numbering system and its own separators.
    assert_eq!(1234567.to_formatted_string(&Locale::ar_EG), "١٬٢٣٤٬٥٦٧");
    assert_eq!(0.5f64.to_formatted_string(&Locale::ar_EG), "٠٫٥");
    assert_eq!(1234567.to_formatted_string(&Locale::bn), "১২,৩৪,৫৬৭");
}

#[test]
fn test_float_special_cases_and_signs() {
    let minus = NumberSymbols::for_locale(Locale::ar_EG).minus_sign;
    assert_eq!(
        f64::NEG_INFINITY.to_formatted_string(&Locale::ar_EG),
        format!("{minus}inf")
    );
    assert_eq!(f64::INFINITY.to_formatted_string(&Locale::en), "inf");
    assert_eq!(f64::NAN.to_formatted_string(&Locale::en), "NaN");
    assert_eq!((-0.0f64).to_formatted_string(&Locale::en), "-0");
}

#[test]
fn test_type_coverage() {
    assert_eq!((-1i8).to_formatted_string(&Locale::en), "-1");
    assert_eq!(
        i128::MIN.to_formatted_string(&Locale::en),
        "-170,141,183,460,469,231,731,687,303,715,884,105,728"
    );
    assert_eq!(
        u128::MAX.to_formatted_string(&Locale::en),
        "340,282,366,920,938,463,463,374,607,431,768,211,455"
    );
    assert_eq!(1.5f32.to_formatted_string(&Locale::en), "1.5");
    // f32 keeps its own shortest representation instead of widening to f64.
    assert_eq!(0.1f32.to_formatted_string(&Locale::en), "0.1");
    assert_eq!(
        1e21f64.to_formatted_string(&Locale::en),
        "1,000,000,000,000,000,000,000"
    );
}

#[test]
fn test_indian_grouping() {
    assert_eq!(10000000.to_formatted_string(&Locale::hi), "1,00,00,000");
    assert_eq!(
        123456.78f64.to_formatted_string(&Locale::en_IN),
        "1,23,456.78"
    );
}

#[test]
fn test_formatter_display_and_reuse() {
    let de = NumberFormatter::new(Locale::de);
    assert_eq!(de.symbols().decimal, ",");
    let line = format!("{} / {}", de.format(1234), de.format(-0.5));
    assert_eq!(line, "1.234 / -0,5");
    // Display honours width via to_string.
    assert_eq!(format!("{:>8}", de.format(1234).to_string()), "   1.234");
}
