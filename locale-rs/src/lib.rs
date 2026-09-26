//! A strongly-typed locale library backed by Unicode CLDR data.
//!
//! [`Locale`] is a plain identifier: an enum with one variant per CLDR
//! locale. Locale data is reached through the types of the feature modules,
//! so the API of `Locale` itself does not depend on which features are on.
//!
//! | Feature | Module | Entry points |
//! | --- | --- | --- |
//! | `nums` | [`nums`] | `NumberFormatter`, `NumberSymbols`, `ToFormattedString` |
//! | `currency` | [`currency`] | `CurrencyFormatter`, `Currency`, `ToCurrencyString` |
//! | `datetime` | [`datetime`] | `DateTimeFormatter`, `DateTime`, `DateSymbols` |
//!
//! ```
//! use locale_rs::Locale;
//!
//! let locale: Locale = "en_gb".parse().unwrap();
//! assert_eq!(locale, Locale::en_GB);
//! assert_eq!(locale.fallback(), Some(Locale::en_001));
//! ```

pub mod error;
pub mod locale;

#[cfg(feature = "currency")]
pub mod currency;
#[cfg(feature = "datetime")]
pub mod datetime;
#[cfg(feature = "nums")]
pub mod nums;

mod data;
#[cfg(any(feature = "nums", feature = "datetime"))]
mod format;

pub use error::LocaleError;
pub use locale::{AVAILABLE_LOCALES, CLDR_VERSION, Locale};

#[cfg(feature = "currency")]
pub use currency::{Currency, CurrencyFormatter, ToCurrencyString};
#[cfg(feature = "datetime")]
pub use datetime::{DateTime, DateTimeFormatter};
#[cfg(feature = "nums")]
pub use nums::{NumberFormatter, ToFormattedString};

#[cfg(test)]
mod test;

/// Runs the README examples as doctests.
#[cfg(all(doctest, feature = "all"))]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
