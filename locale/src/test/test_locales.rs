use crate::{AVAILABLE_LOCALES, Locale};
use std::str::FromStr;

/// Tests that every locale in the CLDR dataset can be converted
/// to a Locale enum and back to the identical string.
#[test]
fn test_all_locales_round_trip() {
    for &locale_str in AVAILABLE_LOCALES.iter() {
        // Test FromStr / try_from
        let locale = Locale::from_str(locale_str)
            .expect(&format!("Failed to parse valid locale: {}", locale_str));

        // Test as_str()
        assert_eq!(locale.as_str(), locale_str);

        // Test Display trait
        assert_eq!(format!("{}", locale), locale_str);

        // Test Into<&'static str>
        let s: &'static str = locale.into();
        assert_eq!(s, locale_str);

        // Test Into<String>
        let string_res: String = locale.into();
        assert_eq!(string_res, locale_str);
    }
}

/// Tests that the &Locale to &'static str conversion works.
#[test]
fn test_reference_conversion() {
    let locale = Locale::from_str(AVAILABLE_LOCALES[0]).unwrap();
    let s: &'static str = (&locale).into();
    assert_eq!(s, AVAILABLE_LOCALES[0]);
}

/// Tests that invalid locale strings return the expected error.
#[test]
fn test_invalid_locale() {
    let invalid_inputs = vec!["", "invalid_locale", "en-US-12345", "123"];

    for input in invalid_inputs {
        let result = Locale::from_str(input);
        assert!(result.is_err(), "Input '{}' should have failed", input);

        // Verifies TryFrom also fails
        let try_result = Locale::try_from(input);
        assert!(try_result.is_err());
    }
}

/// Ensures the Clone, Copy, and Eq traits work as expected.
#[test]
fn test_traits() {
    let loc1 = Locale::from_str(AVAILABLE_LOCALES[0]).unwrap();
    let loc2 = loc1; // Test Copy
    let loc3 = loc1.clone(); // Test Clone

    assert_eq!(loc1, loc2);
    assert_eq!(loc1, loc3);

    // Test Hash (by using it in a set)
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(loc1);
    assert!(set.contains(&loc2));
}

/// Verifies the Debug implementation doesn't panic
#[test]
fn test_debug_print() {
    let loc = Locale::from_str(AVAILABLE_LOCALES[0]).unwrap();
    let debug_str = format!("{:?}", loc);
    assert!(!debug_str.is_empty());
}

#[test]
fn test_fallback_logic() {
    // Test 1: Regional to Base (if both exist in your zip)
    // Note: Replace "en-US" and "en" with locales you know exist in your source
    if let Ok(regional) = Locale::from_str("en-US") {
        if let Some(fallback) = regional.fallback() {
            assert_eq!(fallback.as_str(), "en");
        }
    }

    // Test 2: Base locale should have no fallback
    if let Ok(base) = Locale::from_str("en") {
        assert!(
            base.fallback().is_none(),
            "Base language 'en' should not have a fallback"
        );
    }
}

#[test]
fn test_recursive_fallback() {
    // Test that we can walk up the chain manually
    // e.g., zh-Hant-HK -> zh-Hant -> zh
    let mut current = Locale::from_str("zh-Hant-HK").ok();
    let mut steps = 0;

    while let Some(loc) = current {
        current = loc.fallback();
        steps += 1;
    }

    // If zh-Hant-HK, zh-Hant, and zh all exist, steps should be 3
    assert!(steps >= 1);
}

#[test]
fn test_all_fallbacks_are_valid() {
    for name in AVAILABLE_LOCALES {
        let loc = Locale::from_str(name).unwrap();
        if let Some(fallback) = loc.fallback() {
            // Ensure the fallback string is a prefix of the original
            assert!(name.starts_with(fallback.as_str()));
            // Ensure it's not the same string
            assert_ne!(name, fallback.as_str());
        }
    }
}
