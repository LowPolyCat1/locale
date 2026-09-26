use crate::currency::{Currency, CurrencyFormatter, ToCurrencyString};
use crate::locale::Locale;

#[test]
fn test_placement_and_symbol() {
    assert_eq!(1.99.to_currency(&Locale::en), "$1.99");
    assert_eq!(1.99.to_currency(&Locale::de), "1,99\u{a0}€");
    assert_eq!(1.99.to_currency(&Locale::en_GB), "£1.99");
}

#[test]
fn test_default_currency_follows_region() {
    let code = |l| Currency::default_for(l).to_string();
    assert_eq!(code(Locale::en), "USD");
    assert_eq!(code(Locale::de), "EUR");
    assert_eq!(code(Locale::de_AT), "EUR");
    assert_eq!(code(Locale::de_CH), "CHF");
    assert_eq!(code(Locale::en_GB), "GBP");
    assert_eq!(code(Locale::en_IN), "INR");
    assert_eq!(code(Locale::ja), "JPY");
    assert_eq!(code(Locale::es_419), "USD");
}

#[test]
fn test_whole_amounts_keep_fraction_digits() {
    assert_eq!(100.0.to_currency(&Locale::en), "$100.00");
    assert_eq!(1000.to_currency(&Locale::de), "1.000,00\u{a0}€");
    assert_eq!(0.to_currency(&Locale::en), "$0.00");
}

#[test]
fn test_negative_formatting() {
    assert_eq!((-1.99).to_currency(&Locale::en), "-$1.99");
    assert_eq!((-50.0).to_currency(&Locale::de), "-50,00\u{a0}€");
    // Explicit negative subpattern `¤-#,##0.00`.
    assert_eq!((-5).to_currency(&Locale::de_CH), "CHF-5.00");
    // Amounts that round to zero are not negative.
    assert_eq!((-0.001).to_currency(&Locale::en), "$0.00");
}

#[test]
fn test_large_numbers_and_grouping() {
    assert_eq!(1234567.89.to_currency(&Locale::en), "$1,234,567.89");
    assert_eq!(1234567.89.to_currency(&Locale::de), "1.234.567,89\u{a0}€");
    // Indian grouping from the currency pattern `¤#,##,##0.00`.
    assert_eq!(1234567.89.to_currency(&Locale::en_IN), "₹12,34,567.89");
    assert_eq!(1234567.89.to_currency(&Locale::as_), "₹\u{a0}১২,৩৪,৫৬৭.৮৯");
}

#[test]
fn test_integers_are_exact() {
    // Beyond 2^53 an f64 detour would lose digits.
    assert_eq!(
        9_007_199_254_740_993u64.to_currency(&Locale::en),
        "$9,007,199,254,740,993.00"
    );
    assert_eq!(
        i128::MIN.to_currency_in(&Locale::en, Currency::new("JPY").unwrap()),
        "-¥170,141,183,460,469,231,731,687,303,715,884,105,728"
    );
}

#[test]
fn test_rounding_half_even_on_shortest_decimal() {
    assert_eq!(1.999.to_currency(&Locale::en), "$2.00");
    assert_eq!(1.994.to_currency(&Locale::en), "$1.99");
    assert_eq!(999.995.to_currency(&Locale::en), "$1,000.00");
    assert_eq!(9.5.to_currency(&Locale::ja), "￥10");
    // Ties go to the even digit, as in ICU.
    assert_eq!(0.125.to_currency(&Locale::en), "$0.12");
    assert_eq!(0.135.to_currency(&Locale::en), "$0.14");
    assert_eq!(1.5.to_currency(&Locale::ja), "￥2");
    assert_eq!(2.5.to_currency(&Locale::ja), "￥2");
    // The decimal 1.015 counts, not the f64 just below it.
    assert_eq!(1.015.to_currency(&Locale::en), "$1.02");
    assert_eq!(0.0151.to_currency(&Locale::en), "$0.02");
    assert_eq!(
        1e21.to_currency(&Locale::en),
        "$1,000,000,000,000,000,000,000.00"
    );
    assert_eq!(f64::MIN_POSITIVE.to_currency(&Locale::en), "$0.00");
    assert_eq!(f64::NAN.to_currency(&Locale::en), "$NaN");
    assert_eq!(f64::NEG_INFINITY.to_currency(&Locale::en), "-$inf");
}

#[test]
fn test_explicit_currency() {
    let eur = Currency::new("eur").unwrap();
    assert_eq!(eur.as_str(), "EUR");
    assert_eq!(5.to_currency_in(&Locale::en, eur), "€5.00");
    // CLDR spells the euro out in Swiss German.
    assert_eq!(5.to_currency_in(&Locale::de_CH, eur), "EUR\u{a0}5.00");

    // Fraction digits come from the currency.
    assert_eq!(Currency::new("KWD").unwrap().fraction_digits(), 3);
    assert_eq!(
        1.5.to_currency_in(&Locale::en, Currency::new("KWD").unwrap()),
        "KWD1.500"
    );

    // Unknown codes fall back to the code and two digits.
    let xyz = Currency::new("XYZ").unwrap();
    assert_eq!(xyz.fraction_digits(), 2);
    assert_eq!(1.to_currency_in(&Locale::en, xyz), "XYZ1.00");

    assert!(Currency::new("E1R").is_err());
    assert!("EURO".parse::<Currency>().is_err());
}

#[test]
fn test_symbol_inheritance() {
    // en-AU inherits via en-001 but overrides the symbol of its own dollar.
    assert_eq!(CurrencyFormatter::new(Locale::en_AU).symbol(), "$");
    assert_eq!(
        CurrencyFormatter::with_currency(Locale::en_AU, Currency::new("USD").unwrap()).symbol(),
        "USD"
    );
    assert_eq!(
        CurrencyFormatter::with_currency(Locale::en_001, Currency::new("USD").unwrap()).symbol(),
        "US$"
    );
}

#[test]
fn test_native_digits_everywhere() {
    // Fraction digits are translated too.
    assert_eq!(
        1.99.to_currency(&Locale::ar_EG),
        "\u{200f}١٫٩٩\u{a0}ج.م.\u{200f}"
    );
}

#[test]
fn test_every_locale_formats() {
    for &name in crate::AVAILABLE_LOCALES.iter() {
        let locale: Locale = name.parse().unwrap();
        let formatter = CurrencyFormatter::new(locale);
        let text = formatter.format(-1234.5).to_string();
        assert!(text.contains(formatter.symbol()), "{name}: {text}");
        assert!(!text.contains('¤') && !text.contains('#'), "{name}: {text}");
    }
}
