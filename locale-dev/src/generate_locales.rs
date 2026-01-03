use std::fs;
use std::io::Cursor;
use zip::ZipArchive;

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

pub fn run(
    zip_buffer: Vec<u8>,
    asset_name: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = ZipArchive::new(Cursor::new(zip_buffer))?;
    let mut locales = Vec::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if file.is_dir() && file.name().contains("/main/") {
            let parts: Vec<&str> = file.name().split('/').collect();
            if let Some(idx) = parts.iter().position(|&r| r == "main") {
                if let Some(name) = parts.get(idx + 1) {
                    if !name.is_empty() && !locales.contains(&(*name).to_string()) {
                        locales.push((*name).to_string());
                    }
                }
            }
        }
    }
    locales.sort();

    let mut variants = String::new();
    let mut names = String::new();
    let mut from_str = String::new();
    let mut to_str = String::new();

    for name in &locales {
        let var = sanitize_variant(name);
        variants.push_str(&format!("    {},\n", var));
        names.push_str(&format!("    \"{}\",\n", name));
        from_str.push_str(&format!(
            "            \"{}\" => Ok(Locale::{}),\n",
            name, var
        ));
        to_str.push_str(&format!("            Locale::{} => \"{}\",\n", var, name));
    }

    let code = format!(
        r#"// Auto-generated. DO NOT EDIT.
use std::str::FromStr;
use std::fmt;
use strum_macros::EnumIter;

pub const SOURCE_ASSET: &str = "{asset_name}";
pub const AVAILABLE_LOCALES: [&str; {count}] = [
{names}];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
#[allow(non_camel_case_types)]
pub enum Locale {{
{variants}}}

impl Locale {{
    pub fn as_str(&self) -> &'static str {{
        match self {{
{to_str}        }}
    }}
}}

impl fmt::Display for Locale {{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
        write!(f, "{{}}", self.as_str())
    }}
}}

impl FromStr for Locale {{
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {{
        match s {{
{from_str}            _ => Err(format!("Unknown locale: {{}}", s)),
        }}
    }}
}}

impl From<Locale> for &'static str {{
    fn from(loc: Locale) -> Self {{
        loc.as_str()
    }}
}}

impl Into<String> for Locale {{
    fn into(self) -> String {{
        self.as_str().to_string()
    }}
}}

impl Into<&str> for &'static Locale {{
    fn into(self) -> &'static str {{
        self.as_str()
    }}
}}

"#,
        asset_name = asset_name,
        count = locales.len(),
        names = names,
        variants = variants,
        from_str = from_str,
        to_str = to_str
    );

    fs::write(output_path, code)?;
    println!("Generated {} locales.", locales.len());
    Ok(())
}
