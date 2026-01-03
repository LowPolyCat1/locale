use crate::locale::Locale;
use crate::num_formats::ToFormattedString;

#[test]
fn test_diverse_numerical_symbols() {
    // 1. Different Decimal Separators
    // German uses ',' as decimal. This hits the locale.decimal_separator() branch in impl_float.
    assert_eq!(1.23f64.to_formatted_string(&Locale::de), "1,23");
    // English uses '.'
    assert_eq!(1.23f64.to_formatted_string(&Locale::en), "1.23");

    // 2. Different Grouping Separators
    // German uses '.' as grouping.
    assert_eq!(1000.to_formatted_string(&Locale::de), "1.000");
    // Swiss German often uses '\''
    let gsw_res = 1000.to_formatted_string(&Locale::gsw);
    assert!(gsw_res.contains('\'') || gsw_res.contains(' '));

    // 3. Different Minus Signs
    // Some locales use a specific Unicode minus (U+2212) instead of ASCII hyphen.
    // This exercises the locale.minus_sign() branch in both macros.
    let neg_val = -100;
    let res_ar = neg_val.to_formatted_string(&Locale::ar_EG);
    // Arabic often puts the sign in a different place or uses a different character
    assert!(res_ar.contains(Locale::ar_EG.minus_sign()));
}

#[test]
fn test_native_numbering_systems_exhaustive() {
    // This exercises the 'Some(d)' branch of _translate_digits.
    // ar_EG uses the 'arab' numbering system (١٢٣)
    let val = 1234567;
    let res = val.to_formatted_string(&Locale::ar_EG);

    // Ensure NO ASCII digits remain if the system is fully native
    assert!(!res.contains('1'));
    assert!(!res.contains('2'));

    // This also verifies that the grouping separator used is the one
    // appropriate for that numbering system (e.g., U+066C)
    assert!(res.contains(Locale::ar_EG.grouping_separator()));
}

#[test]
fn test_float_special_cases_and_signs() {
    // Hits the infinite branch with a custom locale minus sign
    let neg_inf = f64::NEG_INFINITY;
    let res = neg_inf.to_formatted_string(&Locale::ar_EG);

    let expected_sign = Locale::ar_EG.minus_sign();
    assert!(res.contains(expected_sign));
    assert!(res.contains("inf"));
}

#[test]
fn test_macro_type_coverage() {
    // Exercise different bit-widths to ensure macro expansion coverage
    // i8 (Smallest signed)
    assert_eq!((-1i8).to_formatted_string(&Locale::en), "-1");
    // u128 (Largest unsigned)
    assert!(1000u128.to_formatted_string(&Locale::en).contains(','));
    // f32 vs f64
    assert_eq!(1.5f32.to_formatted_string(&Locale::en), "1.5");
    assert_eq!(1.5f64.to_formatted_string(&Locale::en), "1.5");
}

#[test]
fn test_grouping_exhaustion_detailed() {
    // Test the 'size_idx' logic for a locale that might have more than 2 grouping steps
    // If a locale had [3, 2, 1], this would be vital.
    // Most are [3] or [3, 2].
    let hi_ = &Locale::hi;
    // 1,00,00,000 (The '2' is repeated indefinitely after the first '3')
    assert_eq!(10000000.to_formatted_string(hi_), "1,00,00,000");
}
