pub mod cldr;
pub mod download_latest;
pub mod emit;
pub mod error;
pub mod format;
pub mod patterns;
pub mod readme;
pub mod version;

#[cfg(test)]
mod test;

use cldr::Cldr;
use error::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

pub fn sanitize_variant(name: &str) -> String {
    let variant = name.replace("-", "_");
    if RUST_KEYWORDS.contains(&variant.as_str()) {
        format!("{}_", variant)
    } else {
        variant
    }
}

/// Generates every data module of `locale-rs` into `data_dir`
/// (`locale-rs/src/data`) and formats them. Returns the written files.
///
/// The emitters run in parallel, as does rustfmt.
pub fn generate(cldr: &Cldr, cldr_version: &str, data_dir: &Path) -> Result<Vec<PathBuf>> {
    type Emitter = fn(&Cldr, &str) -> Result<emit::RustFile>;
    let emitters: [(&str, Emitter); 4] = [
        ("locales.rs", emit::locales::emit),
        ("numbers.rs", emit::numbers::emit),
        ("dates.rs", emit::dates::emit),
        ("currency.rs", emit::currency::emit),
    ];
    let written = emitters
        .par_iter()
        .map(|(name, emit)| emit(cldr, cldr_version)?.write(&data_dir.join(name)))
        .collect::<Result<Vec<_>>>()?;
    format::format_generated_code(&written)?;
    tracing::info!(
        "Generated {} locales into {}",
        cldr.locales.len(),
        data_dir.display()
    );
    Ok(written)
}
