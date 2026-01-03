use serde_json::Value;
use std::fs;
use std::io::Cursor;
use zip::ZipArchive;

use crate::sanitize_variant;

fn detect_all_groupings(pattern: &str) -> Vec<usize> {
    let integer_part = pattern.split('.').next().unwrap_or(pattern);
    let clean: String = integer_part
        .chars()
        .filter(|&c| c == '#' || c == '0' || c == ',')
        .collect();

    let parts: Vec<&str> = clean.split(',').collect();
    if parts.len() <= 1 {
        return vec![0];
    }

    let mut sizes = Vec::new();
    if let Some(primary) = parts.last() {
        sizes.push(primary.len());
    }
    if parts.len() > 2 {
        if let Some(secondary) = parts.get(parts.len() - 2) {
            sizes.push(secondary.len());
        }
    }

    if sizes.len() > 1 && sizes[0] == sizes[1] {
        sizes.truncate(1);
    }

    sizes
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
        let mut grouping_sizes = vec![3];

        let json_path = format!("cldr-numbers-full/main/{}/numbers.json", name);
        if let Ok(mut file) = archive.by_name(&json_path) {
            let json: Value = serde_json::from_reader(&mut file)?;
            let numbers = &json["main"][name]["numbers"];
            let symbols = &numbers["symbols-numberSystem-latn"];
            if let Some(d) = symbols["decimal"].as_str() {
                decimal = d.to_string();
            }
            if let Some(g) = symbols["group"].as_str() {
                group = g.to_string();
            }
            if let Some(pattern) = numbers["decimalFormats-numberSystem-latn"]["standard"].as_str()
            {
                grouping_sizes = detect_all_groupings(pattern);
            }
        }

        dec_sep_arms.push_str(&format!(
            "            Locale::{} => \"{}\",\n",
            var, decimal
        ));
        grp_sep_arms.push_str(&format!("            Locale::{} => \"{}\",\n", var, group));
        grp_size_arms.push_str(&format!(
            "            Locale::{} => &{:?},\n",
            var, grouping_sizes
        ));
    }

    let code = format!(
        r#"// Auto-generated. DO NOT EDIT.
use crate::locale::Locale;

impl Locale {{
    pub fn decimal_separator(&self) -> &'static str {{
        match self {{ {dec_sep_arms} }}
    }}

    pub fn grouping_separator(&self) -> &'static str {{
        match self {{ {grp_sep_arms} }}
    }}

    pub fn grouping_sizes(&self) -> &'static [usize] {{
        match self {{ {grp_size_arms} }}
    }}
}}

pub trait ToFormattedString {{
    fn to_formatted_string(&self, locale: &Locale) -> String;
}}

fn _format_int_str(numeric_part: &str, locale: &Locale) -> String {{
    let sizes = locale.grouping_sizes();
    let separator = locale.grouping_separator();

    if sizes.is_empty() || sizes[0] == 0 || numeric_part.len() <= sizes[0] {{
        return numeric_part.to_string();
    }}

    let mut result = Vec::with_capacity(numeric_part.len() + 8);
    let bytes = numeric_part.as_bytes();

    let mut i = 0;         // Digits placed in CURRENT group
    let mut size_idx = 0;   // Current index in the sizes array

    for &byte in bytes.iter().rev() {{
        let current_target_size = sizes[size_idx];

        // If the current group is full, insert separator and move to next size
        if i == current_target_size {{
            for b in separator.as_bytes().iter().rev() {{
                result.push(*b);
        }}
            i = 0;
            if size_idx < sizes.len() - 1 {{
                size_idx += 1;
        }}
    }}

        result.push(byte);
        i += 1;
    }}

    result.reverse();
    unsafe {{ String::from_utf8_unchecked(result) }}
}}


macro_rules! impl_int {{
    ($($t:ty),*) => {{
        $(
            impl ToFormattedString for $t {{
                fn to_formatted_string(&self, locale: &Locale) -> String {{
                    let s = self.to_string();
                    if s.starts_with('-') {{
                        format!("-{{}}", _format_int_str(&s[1..], locale))
                    }} else {{
                        _format_int_str(&s, locale)
                    }}
                }}
            }}
        )*
    }};
}}

impl_int!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);

macro_rules! impl_float {{
    ($($t:ty),*) => {{
        $(
            impl ToFormattedString for $t {{
                fn to_formatted_string(&self, locale: &Locale) -> String {{
                    if self.is_nan() {{ return "NaN".to_string(); }}
                    if self.is_infinite() {{ return "inf".to_string(); }}

                    let s = format!("{{}}", self);
                    if let Some(pos) = s.find('.') {{
                        let (int_part, frac_part) = s.split_at(pos);
                        let formatted_int = if int_part.starts_with('-') {{
                            format!("-{{}}", _format_int_str(&int_part[1..], locale))
                        }} else {{
                            _format_int_str(int_part, locale)
                        }};
                        // frac_part[1..] skips the original '.'
                        format!("{{}}{{}}{{}}", formatted_int, locale.decimal_separator(), &frac_part[1..])
                    }} else {{
                        if s.starts_with('-') {{
                            format!("-{{}}", _format_int_str(&s[1..], locale))
                        }} else {{
                            _format_int_str(&s, locale)
                        }}
                    }}
                }}
            }}
        )*
    }};
}}

impl_float!(f32, f64);
"#,
        dec_sep_arms = dec_sep_arms,
        grp_sep_arms = grp_sep_arms,
        grp_size_arms = grp_size_arms
    );

    fs::write(output_path, code)?;
    Ok(())
}
