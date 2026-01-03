use serde_json::Value;
use std::fs;
use std::io::Cursor;
use zip::ZipArchive;

use crate::sanitize_variant;

fn parse_grouping_size(pattern: &str) -> usize {
    let integer_part = pattern.split('.').next().unwrap_or(pattern);
    if let Some(last_comma) = integer_part.rfind(',') {
        return integer_part.len() - last_comma - 1;
    }
    3
}
pub fn run(
    zip_buffer: Vec<u8>,
    _asset_name: &str,
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

    let mut dec_sep_arms = String::new();
    let mut grp_sep_arms = String::new();
    let mut grp_size_arms = String::new();

    for name in &locales {
        let var = sanitize_variant(name);
        let mut decimal = ".".to_string();
        let mut group = ",".to_string();
        let mut group_size = 3;

        let json_path = format!("cldr-numbers-full/main/{}/numbers.json", name);
        if let Ok(mut file) = archive.by_name(&json_path) {
            if let Ok(json) = serde_json::from_reader::<_, Value>(&mut file) {
                let numbers = &json["main"][name]["numbers"];
                let symbols = &numbers["symbols-numberSystem-latn"];
                if let Some(d) = symbols["decimal"].as_str() {
                    decimal = d.to_string();
                }
                if let Some(g) = symbols["group"].as_str() {
                    group = g.to_string();
                }
                if let Some(pattern) =
                    numbers["decimalFormats-numberSystem-latn"]["standard"].as_str()
                {
                    group_size = parse_grouping_size(pattern);
                }
            }
        }

        dec_sep_arms.push_str(&format!(
            "            Locale::{} => \"{}\",\n",
            var, decimal
        ));
        grp_sep_arms.push_str(&format!("            Locale::{} => \"{}\",\n", var, group));
        grp_size_arms.push_str(&format!("            Locale::{} => {},\n", var, group_size));
    }

    let code = format!(
        r#"// Auto-generated. DO NOT EDIT.
use crate::locale::Locale;

impl Locale {{
    pub fn decimal_separator(&self) -> &'static str {{
        match self {{
{dec_sep_arms}        }}
    }}

    pub fn grouping_separator(&self) -> &'static str {{
        match self {{
{grp_sep_arms}        }}
    }}

    pub fn grouping_size(&self) -> usize {{
        match self {{
{grp_size_arms}        }}
    }}
}}

pub trait ToFormattedString {{
    fn to_formatted_string(&self, locale: &Locale) -> String;
}}

macro_rules! impl_int_format {{
    ($($t:ty),*) => {{
        $(
            impl ToFormattedString for $t {{
                fn to_formatted_string(&self, locale: &Locale) -> String {{
                    let s = self.to_string();
                    let is_negative = s.starts_with('-');
                    let numeric_part = if is_negative {{ &s[1..] }} else {{ &s }};

                    let group_size = locale.grouping_size();
                    let separator = locale.grouping_separator();

                    if group_size == 0 {{ return s; }}

                    let mut result = String::with_capacity(s.len() + (s.len() / group_size));
                    if is_negative {{ result.push('-'); }}

                    let bytes = numeric_part.as_bytes();
                    let len = bytes.len();

                    for (i, byte) in bytes.iter().enumerate() {{
                        result.push(*byte as char);
                        let remaining = len - i - 1;
                        if remaining > 0 && remaining % group_size == 0 {{
                            result.push_str(separator);
                        }}
                    }}
                    result
                }}
            }}
        )*
    }};
}}

impl_int_format!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_num_formatting() {{
        // Test English (Standard 3-digit grouping)
        assert_eq!(1000000u32.to_formatted_string(&Locale::en), "1,000,000");

        // Test locales that might use different separators (like French space or German dot)
        let thousand = 1000i32;
        let large = 1234567i32;

        assert_eq!(thousand.to_formatted_string(&Locale::en), "1,000");

        // Negative test
        assert_eq!((-1000i32).to_formatted_string(&Locale::en), "-1,000");
    }}
}}
"#,
        dec_sep_arms = dec_sep_arms,
        grp_sep_arms = grp_sep_arms,
        grp_size_arms = grp_size_arms
    );

    fs::write(output_path, code)?;
    Ok(())
}
