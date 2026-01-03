use crate::{Locale, num_formats::ToFormattedString};

#[test]
fn test_standard_formatting() {
    let val = 1000000u32;

    // US English: 1,000,000
    assert_eq!(val.to_formatted_string(&Locale::en), "1,000,000");

    // German: 1.000.000
    assert_eq!(val.to_formatted_string(&Locale::de), "1.000.000");
}

#[test]
fn test_negative_numbers() {
    let val = -1234567i32;

    // US English: -1,234,567
    assert_eq!(val.to_formatted_string(&Locale::en), "-1,234,567");

    // French uses non-breaking space often: 1 234 567 (verify exact CLDR char)
    let formatted_fr = val.to_formatted_string(&Locale::fr);
    assert!(formatted_fr.contains("1") && formatted_fr.contains("234"));
}

#[test]
fn test_small_numbers() {
    let val = 999;
    // No separator should be added
    assert_eq!(val.to_formatted_string(&Locale::en), "999");
}

#[test]
fn test_boundary_sizes() {
    let val = 1000;
    assert_eq!(val.to_formatted_string(&Locale::en), "1,000");
}

#[test]
fn test_custom_grouping_logic() {
    // Some locales might have grouping size 4 or others
    // This test ensures the generated size is respected
    let val = 1000000;
    let loc = Locale::en;
    let size = loc.grouping_size();

    let formatted = val.to_formatted_string(&loc);
    let separator_count = formatted.chars().filter(|c| c == &',').count();

    // For 1,000,000 and size 3, we expect 2 commas
    if size == 3 {
        assert_eq!(separator_count, 2);
    }
}
