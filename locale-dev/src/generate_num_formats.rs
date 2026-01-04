use serde_json::Value;
use std::collections::HashMap;
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
    if parts.len() > 2
        && let Some(secondary) = parts.get(parts.len() - 2) {
            sizes.push(secondary.len());
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

    let mut system_digit_map: HashMap<String, [char; 10]> = HashMap::new();
    if let Ok(mut file) = archive.by_name("cldr-core/supplemental/numberingSystems.json") {
        let json: Value = serde_json::from_reader(&mut file)?;
        if let Some(systems) = json["supplemental"]["numberingSystems"].as_object() {
            for (name, data) in systems {
                if data["_type"].as_str() == Some("numeric")
                    && let Some(digits_str) = data["_digits"].as_str() {
                        let chars: Vec<char> = digits_str.chars().collect();
                        if chars.len() == 10 {
                            let mut arr = ['0'; 10];
                            arr.copy_from_slice(&chars[..10]);
                            system_digit_map.insert(name.to_string(), arr);
                        }
                    }
            }
        }
    }

    let mut locales = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if file.name().contains("/main/") && file.is_dir() {
            let parts: Vec<&str> = file.name().split('/').collect();
            if let Some(idx) = parts.iter().position(|&r| r == "main")
                && let Some(name) = parts.get(idx + 1)
                    && !name.is_empty() && !locales.contains(&(*name).to_string()) {
                        locales.push((*name).to_string());
                    }
        }
    }
    locales.sort();

    let mut dec_sep_arms = String::new();
    let mut grp_sep_arms = String::new();
    let mut grp_size_arms = String::new();
    let mut digit_arms = String::new();
    let mut minus_arms = String::new();

    for name in &locales {
        let var = sanitize_variant(name);
        let mut decimal = ".".to_string();
        let mut group = ",".to_string();
        let mut minus = "-".to_string();
        let mut grouping_sizes = vec![3];
        let mut digit_set_str = "None".to_string();

        let json_path = format!("cldr-numbers-full/main/{}/numbers.json", name);
        if let Ok(mut file) = archive.by_name(&json_path) {
            let json: Value = serde_json::from_reader(&mut file)?;
            let numbers = &json["main"][name]["numbers"];

            let system = numbers["defaultNumberingSystem"].as_str().unwrap_or("latn");
            let symbols_key = format!("symbols-numberSystem-{}", system);
            let symbols = &numbers[&symbols_key];

            if let Some(d) = symbols["decimal"].as_str() {
                decimal = d.to_string();
            }
            if let Some(g) = symbols["group"].as_str() {
                group = g.to_string();
            }
            if let Some(m) = symbols["minusSign"].as_str() {
                minus = m.to_string();
            }

            if system != "latn"
                && let Some(digits) = system_digit_map.get(system) {
                    digit_set_str = format!("Some({:?})", digits);
                }

            let format_key = format!("decimalFormats-numberSystem-{}", system);
            if let Some(pattern) = numbers[format_key]["standard"].as_str() {
                grouping_sizes = detect_all_groupings(pattern);
            }
        }

        dec_sep_arms.push_str(&format!(
            "            Locale::{} => \"{}\",\n",
            var, decimal
        ));
        grp_sep_arms.push_str(&format!("            Locale::{} => \"{}\",\n", var, group));
        minus_arms.push_str(&format!("            Locale::{} => \"{}\",\n", var, minus));
        grp_size_arms.push_str(&format!(
            "            Locale::{} => &{:?},\n",
            var, grouping_sizes
        ));
        digit_arms.push_str(&format!(
            "            Locale::{} => {},\n",
            var, digit_set_str
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

    pub fn minus_sign(&self) -> &'static str {{
        match self {{ {minus_arms} }}
    }}

    pub fn digits(&self) -> Option<[char; 10]> {{
        match self {{ {digit_arms} }}
    }}
}}

pub trait ToFormattedString {{
    fn to_formatted_string(&self, locale: &Locale) -> String;
}}

/// Translates ASCII digits 0-9 into the locale's native numbering system.
pub fn translate_digits(input: String, locale: &Locale) -> String {{
    match locale.digits() {{
        Some(d) => input.chars().map(|c| {{
            if c.is_ascii_digit() {{
                let idx = (c as u8 - b'0') as usize;
                d[idx]
            }} else {{
                c
            }}
        }}).collect(),
        None => input,
    }}
}}

/// Formats the integer portion of a number with grouping separators.
fn _format_int_str(numeric_part: &str, locale: &Locale) -> String {{
    let sizes = locale.grouping_sizes();
    let separator = locale.grouping_separator();

    if sizes.is_empty() || sizes[0] == 0 || numeric_part.len() <= sizes[0] {{
        return numeric_part.to_string();
    }}

    let mut result = Vec::with_capacity(numeric_part.len() + 32);
    let bytes = numeric_part.as_bytes();

    let mut i = 0;
    let mut size_idx = 0;

    for &byte in bytes.iter().rev() {{
        let current_target_size = sizes[size_idx];

        if i == current_target_size {{
            // Append separator in reverse (for later full-string reversal)
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
                    let (is_neg, abs_str) = if s.starts_with('-') {{ (true, &s[1..]) }} else {{ (false, &s[..]) }};

                    let formatted = _format_int_str(abs_str, locale);
                    let res = if is_neg {{
                        format!("{{}}{{}}", locale.minus_sign(), formatted)
                    }} else {{
                        formatted
                    }};
                    translate_digits(res, locale)
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
                    if self.is_infinite() {{
                        return if self.is_sign_positive() {{ "inf".to_string() }} else {{ format!("{{}}inf", locale.minus_sign()) }};
                    }}

                    let s = format!("{{}}", self);
                    let (is_neg, s_abs) = if s.starts_with('-') {{ (true, &s[1..]) }} else {{ (false, &s[..]) }};

                    let res = if let Some(pos) = s_abs.find('.') {{
                        let (int_part, frac_with_dot) = s_abs.split_at(pos);
                        let formatted_int = _format_int_str(int_part, locale);
                        format!("{{}}{{}}{{}}", formatted_int, locale.decimal_separator(), &frac_with_dot[1..])
                    }} else {{
                        _format_int_str(s_abs, locale)
                    }};

                    let final_str = if is_neg {{ format!("{{}}{{}}", locale.minus_sign(), res) }} else {{ res }};
                    translate_digits(final_str, locale)
                }}
            }}
        )*
    }};
}}

impl_float!(f32, f64);
"#,
        dec_sep_arms = dec_sep_arms,
        grp_sep_arms = grp_sep_arms,
        grp_size_arms = grp_size_arms,
        minus_arms = minus_arms,
        digit_arms = digit_arms
    );

    fs::write(output_path, code)?;
    Ok(())
}
