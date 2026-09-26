use thiserror::Error;

/// Everything that can go wrong while generating code or syncing READMEs.
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Zip archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON error in {path}: {source}")]
    Json {
        path: String,
        source: serde_json::Error,
    },
    #[error("TOML error: {0}")]
    Toml(#[from] toml_edit::TomlError),
    #[error("Generated code does not parse: {0}")]
    Syntax(#[from] syn::Error),
    #[error("Invalid CLDR data: {0}")]
    Cldr(String),
    #[error("rustfmt failed: {0}")]
    Rustfmt(String),
    #[error("Invalid version: {0}")]
    Version(String),
    #[error("{0}")]
    Readme(String),
    #[error("{0}")]
    Usage(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
