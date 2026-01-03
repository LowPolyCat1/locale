use std::str::FromStr;

#[cfg(feature = "strum")]
use strum::IntoEnumIterator;

use crate::{AVAILABLE_LOCALES, Locale};

#[test]
fn convert_from_string_to_enum() {
    for locale_str in AVAILABLE_LOCALES {
        Locale::from_str(locale_str).unwrap();
    }
}

#[test]
#[cfg(feature = "strum")]
fn convert_from_enum_to_string() {
    for locale_enum in Locale::iter() {
        locale_enum.as_str();
        let _locale_str: &str = locale_enum.into();
        let _locale_str: String = locale_enum.into();
    }
}
