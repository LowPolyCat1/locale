use crate::{AVAILABLE_LOCALES, Locale};
use std::str::FromStr;
use strum::IntoEnumIterator;

#[test]

fn test_strum_iter_coverage() {
    // Iterate through every single variant defined in the enum
    for locale in Locale::iter() {
        let s = locale.as_str();

        // 1. Ensure it's not empty
        assert!(
            !s.is_empty(),
            "Locale variant {:?} produced empty string",
            locale
        );

        // 2. Ensure the string round-trips back to the same enum variant
        let parsed = Locale::from_str(s).expect("Variant as_str() failed to parse");
        assert_eq!(parsed, locale);
    }
}

#[test]
fn test_available_locales_matches_enum_count() {
    // If strum is enabled, we can verify our static array is actually complete
    #[cfg(feature = "strum")]
    {
        let enum_count = Locale::iter().count();
        assert_eq!(
            enum_count,
            AVAILABLE_LOCALES.len(),
            "The AVAILABLE_LOCALES array length does not match the number of Enum variants!"
        );
    }
}
