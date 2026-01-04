pub mod download_latest;
pub mod error;
pub mod format;
pub mod generate_currency_formatting;
pub mod generate_datetime_formatting;
pub mod generate_locales;
pub mod generate_num_formats;

#[cfg(test)]
mod test;

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
