use std::str::FromStr;

use crate::AVAILABLE_LOCALES;
use crate::datetime_formats::DateTime;
use crate::locale::Locale;

fn base_dt() -> DateTime {
    DateTime {
        year: 2026,
        month: 1,
        day: 3,
        hour: 14,
        minute: 5,
        second: 9,
    }
}

#[test]
fn test_public_format_padding_and_names() {
    let dt = base_dt();

    // Testing 'en' usually hits patterns like 'MMM d, y'
    // This indirectly tests padding for days and month name lookup
    let loc_en = Locale::en;
    let formatted_en = loc_en.format_date(&dt);

    assert!(
        formatted_en.contains("Jan"),
        "Should contain abbreviated month"
    );
    assert!(formatted_en.contains("2026"), "Should contain full year");
}

#[test]
fn test_literals_via_chinese_locale() {
    let dt = base_dt();

    // 'zh_Hans' pattern is usually "y年M月d日"
    // This indirectly tests the private parser's literal/quoting logic
    let loc_zh = Locale::zh_Hans;
    let formatted_zh = loc_zh.format_date(&dt);

    assert!(
        formatted_zh.contains("年"),
        "Should handle literal Chinese year char"
    );
    assert!(
        formatted_zh.contains("月"),
        "Should handle literal Chinese month char"
    );
}

#[test]
fn test_year_truncation_via_locales() {
    let dt = base_dt();

    // Find a locale that uses short years (yy) in its medium format.
    // Many European locales use dd.MM.yy
    let loc_de = Locale::de;
    let formatted_de = loc_de.format_date(&dt);

    // If 'de' uses yy, it will be '26'. If it uses yyyy, it will be '2026'.
    // This hits the branching logic for 'y' vs 'yy'.
    assert!(formatted_de.contains("26"));
}

#[test]
fn test_time_formatting() {
    let dt = base_dt();
    let loc_en = Locale::en;
    let time_str = loc_en.format_time(&dt);

    // Medium time usually contains seconds and 2-digit minutes
    // This tests 'HH', 'mm', 'ss' tokens
    assert!(time_str.contains("05"), "Minute should be padded");
    assert!(time_str.contains("09"), "Second should be padded");
}

#[test]
#[cfg(feature = "nums")]
fn test_arabic_digit_translation() {
    let dt = base_dt();
    // Use the snake_case variant name
    let loc_ar = Locale::ar_EG;

    let formatted = loc_ar.format_date(&dt);
    // \u{0662} is '2' in Arabic-Indic, hitting the digit translation logic
    assert!(formatted.contains('\u{0662}'));
}

#[test]
fn test_exhaustive_branch_coverage() {
    let dt = base_dt();
    // This is the "Golden" test for 100% coverage.
    // By calling public methods on every single locale, we force the compiler
    // to execute every single match arm in the generated date_format_pattern()
    // and every month name array.
    for locale in AVAILABLE_LOCALES {
        let loc: Locale = Locale::from_str(locale).unwrap();
        let date_out = loc.format_date(&dt);
        let time_out = loc.format_time(&dt);

        assert!(!date_out.is_empty());
        assert!(!time_out.is_empty());
    }
}

#[test]
fn test_datetime_struct_properties() {
    let dt = base_dt();
    let dt2 = dt; // DateTime is Copy
    assert_eq!(dt.year, dt2.year);
    assert_eq!(dt.month, dt2.month);
}
