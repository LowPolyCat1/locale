use thiserror::Error;

/// Errors returned by `locale-rs`.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocaleError {
    /// The string is not the identifier of a known locale.
    #[error("Unknown locale identifier: '{0}'")]
    UnknownLocale(String),
    /// The string is not a three-letter ISO 4217 currency code.
    #[error("Invalid currency code: '{0}'")]
    InvalidCurrency(String),
    /// A date or time field is out of range.
    #[error("Invalid {field}: {value}")]
    InvalidDateTime {
        /// Name of the offending field, e.g. `"month"`.
        field: &'static str,
        /// The rejected value.
        value: i64,
    },
}
