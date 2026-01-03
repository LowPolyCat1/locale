use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LocaleError {
    #[error("Unknown Error Occured: '{0}'")]
    Unknown(String),
    #[error("Unknown locale identifier: '{0}'")]
    UnknownLocale(String),
}
