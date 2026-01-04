use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use zip::ZipArchive;

use crate::sanitize_variant;

pub fn run(
    zip_buffer: Vec<u8>,
    _asset_name: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = ZipArchive::new(Cursor::new(zip_buffer))?;

    // 1. Load Territory -> Currency mapping from supplemental data
    let mut region_to_currency = HashMap::new();
    if let Ok(mut file) = archive.by_name("cldr-core/supplemental/currencyData.json") {
        let json: Value = serde_json::from_reader(&mut file)?;
        if let Some(regions) = json["supplemental"]["currencyData"]["region"].as_object() {
            for (region_code, currencies) in regions {
                if let Some(cur_list) = currencies.as_array() {
                    for entry in cur_list {
                        let cur_code = entry.as_object().and_then(|m| m.keys().next());
                        if let Some(code) = cur_code
                            && entry[code]["_to"].is_null() {
                                region_to_currency.insert(region_code.clone(), code.clone());
                                break;
                            }
                    }
                }
            }
        }
    }

    // 2. Load Language -> Likely Subtag (to map 'en' to 'US', 'de' to 'DE')
    let mut lang_to_region = HashMap::new();
    if let Ok(mut file) = archive.by_name("cldr-core/supplemental/likelySubtags.json") {
        let json: Value = serde_json::from_reader(&mut file)?;
        if let Some(subtags) = json["supplemental"]["likelySubtags"].as_object() {
            for (lang_key, full_locale) in subtags {
                if let Some(full_str) = full_locale.as_str() {
                    let parts: Vec<&str> = full_str.split('-').collect();
                    if parts.len() >= 3 {
                        lang_to_region.insert(lang_key.clone(), parts[2].to_string());
                    }
                }
            }
        }
    }

    // 3. Process Locales
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

    let mut pattern_arms = String::new();
    let mut symbol_arms = String::new();

    for name in &locales {
        let var = sanitize_variant(name);

        // Resolve Currency Code
        let region = lang_to_region
            .get(name)
            .cloned()
            .unwrap_or_else(|| "US".to_string());
        let currency_code = region_to_currency
            .get(&region)
            .cloned()
            .unwrap_or_else(|| "USD".to_string());

        // Resolve Symbol
        let mut symbol = currency_code.clone();
        let cur_path = format!("cldr-numbers-full/main/{}/currencies.json", name);
        if let Ok(mut file) = archive.by_name(&cur_path) {
            let json: Value = serde_json::from_reader(&mut file)?;
            if let Some(s) =
                json["main"][name]["numbers"]["currencies"][&currency_code]["symbol"].as_str()
            {
                symbol = s.to_string();
            }
        }

        // Resolve Pattern
        let mut pattern = "¤#,##0.00".to_string();
        let num_path = format!("cldr-numbers-full/main/{}/numbers.json", name);
        if let Ok(mut file) = archive.by_name(&num_path) {
            let json: Value = serde_json::from_reader(&mut file)?;
            let numbers = &json["main"][name]["numbers"];
            let system = numbers["defaultNumberingSystem"].as_str().unwrap_or("latn");
            let format_key = format!("currencyFormats-numberSystem-{}", system);
            if let Some(p) = numbers[format_key]["standard"].as_str() {
                pattern = p.to_string();
            }
        }

        pattern_arms.push_str(&format!("            Locale::{} => {:?},\n", var, pattern));
        symbol_arms.push_str(&format!("            Locale::{} => {:?},\n", var, symbol));
    }

    let code = format!(
        r#"// Auto-generated. DO NOT EDIT.
use crate::locale::Locale;
use crate::num_formats::ToFormattedString;

impl Locale {{
    pub fn currency_standard_pattern(&self) -> &'static str {{
        match self {{
{pattern_arms}        }}
    }}

    pub fn default_currency_symbol(&self) -> &'static str {{
        match self {{
{symbol_arms}        }}
    }}

    pub fn format_currency<T: Into<f64>>(&self, value: T) -> String {{
        let val: f64 = value.into();
        let symbol = self.default_currency_symbol();
        let pattern = self.currency_standard_pattern();

        let is_negative = val < 0.0;

        // 1. Round to 2 decimal places immediately to prevent precision drift (e.g., 1.999 -> 2.00)
        let abs_val = (val.abs() * 100.0).round() / 100.0;

        // 2. Split after rounding
        let int_part_val = abs_val.floor() as i128;
        let fract_val = ((abs_val - abs_val.floor()) * 100.0).round() as i32;

        let int_part_str = int_part_val.to_formatted_string(self);

        let num_str = if fract_val == 0 {{
            format!("{{}},-", int_part_str)
        }} else {{
            format!("{{}}{{}}{{:02}}", int_part_str, self.decimal_separator(), fract_val)
        }};

        // 3. Apply pattern
        // \u{{00a4}} = ¤ (Currency Placeholder)
        // \x23 = # (Avoids Rust 2024 reserved multi-hash tokens)
        let mut result = pattern
            .replace('\u{{00a4}}', symbol)
            .replace("\x23,\x23\x230.00", &num_str)
            .replace("\x23,\x23\x230", &num_str);

        if is_negative {{
            format!("{{}}{{}}", self.minus_sign(), result)
        }} else {{
            result
        }}
    }}
}}

pub trait ToCurrencyString {{
    fn to_currency(&self, locale: &Locale) -> String;
}}

macro_rules! impl_currency {{
    ($($t:ty),*) => {{
        $(
            impl ToCurrencyString for $t {{
                fn to_currency(&self, locale: &Locale) -> String {{
                    locale.format_currency(*self as f64)
                }}
            }}
        )*
    }};
}}

impl_currency!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, isize, usize);
"#,
        pattern_arms = pattern_arms,
        symbol_arms = symbol_arms
    );

    fs::write(output_path, code)?;
    Ok(())
}
