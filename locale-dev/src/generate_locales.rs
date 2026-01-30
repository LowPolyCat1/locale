use std::fs;
use std::io::Cursor;
use zip::ZipArchive;

use crate::sanitize_variant;

pub fn run(
    zip_buffer: Vec<u8>,
    asset_name: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("generating locales");
    let mut archive = ZipArchive::new(Cursor::new(zip_buffer))?;
    let mut locales = Vec::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if file.is_dir() && file.name().contains("/main/") {
            let parts: Vec<&str> = file.name().split('/').collect();
            if let Some(idx) = parts.iter().position(|&r| r == "main")
                && let Some(name) = parts.get(idx + 1)
                    && !name.is_empty() && !locales.contains(&(*name).to_string()) {
                        locales.push((*name).to_string());
                    }
        }
    }
    locales.sort();

    let mut variants = String::new();
    let mut names = String::new();
    let mut from_str = String::new();
    let mut to_str = String::new();
    let mut fallbacks = String::new();
    let mut lang_code_arms = String::new();
    let mut region_code_arms = String::new();

    for name in &locales {
        let var = sanitize_variant(name);

        variants.push_str(&format!("    {},\n", var));
        names.push_str(&format!("    \"{}\",\n", name));

        from_str.push_str(&format!(
            "            \"{}\" => Ok(Locale::{}),\n",
            name, var
        ));

        to_str.push_str(&format!("            Locale::{} => \"{}\",\n", var, name));

        let mut fallback_found = false;
        if let Some(idx) = name.rfind('-') {
            let parent_name = &name[..idx];
            if locales.contains(&parent_name.to_string()) {
                let parent_var = sanitize_variant(parent_name);
                fallbacks.push_str(&format!(
                    "            Locale::{} => Some(Locale::{}),\n",
                    var, parent_var
                ));
                fallback_found = true;
            }
        }

        if !fallback_found {
            fallbacks.push_str(&format!("            Locale::{} => None,\n", var));
        }

        // Generate language and region code extraction
        let parts: Vec<&str> = name.split('-').collect();
        let lang = parts.first().unwrap_or(&"");
        let region = if parts.len() > 1 {
            // Region is typically the last part if it's 2 uppercase letters
            let last = parts.last().unwrap_or(&"");
            if last.len() == 2 && last.chars().all(|c| c.is_ascii_uppercase()) {
                last.to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        lang_code_arms.push_str(&format!(
            "            Locale::{} => \"{}\",\n",
            var, lang
        ));

        if region.is_empty() {
            region_code_arms.push_str(&format!("            Locale::{} => None,\n", var));
        } else {
            region_code_arms.push_str(&format!(
                "            Locale::{} => Some(\"{}\"),\n",
                var, region
            ));
        }
    }

    let code = format!(
        r#"// Auto-generated. DO NOT EDIT.
use std::str::FromStr;
use std::fmt;
use crate::error::LocaleError;

#[cfg(feature = "strum")]
use strum_macros::EnumIter;

pub const SOURCE_ASSET: &str = "{asset_name}";
pub const AVAILABLE_LOCALES: [&str; {count}] = [
{names}];

#[cfg_attr(feature = "strum", derive(EnumIter))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum Locale {{
{variants}}}

impl Locale {{
    /// Returns the string representation of the locale.
    pub fn as_str(&self) -> &'static str {{
        match self {{
{to_str}        }}
    }}

    /// Returns the next best fallback locale by stripping subtags.
    /// Example: Locale::en_GB.fallback() -> Some(Locale::en)
    pub fn fallback(&self) -> Option<Self> {{
        match self {{
{fallbacks}        }}
    }}

    /// Parses a locale string with flexible formatting.
    /// Accepts both hyphens and underscores, and is case-insensitive.
    ///
    /// # Examples
    /// ```
    /// use locale_rs::Locale;
    ///
    /// assert_eq!(Locale::from_flexible("en-GB"), Ok(Locale::en_GB));
    /// assert_eq!(Locale::from_flexible("en_gb"), Ok(Locale::en_GB));
    /// assert_eq!(Locale::from_flexible("EN-gb"), Ok(Locale::en_GB));
    /// ```
    pub fn from_flexible(s: &str) -> Result<Self, LocaleError> {{
        let normalized = s.replace('_', "-").to_lowercase();
        Self::from_str(&normalized)
    }}

    /// Finds the best matching locale from a list of available locales.
    /// Uses fallback chain matching to find the closest match.
    ///
    /// # Examples
    /// ```
    /// use locale_rs::Locale;
    ///
    /// let available = vec![Locale::en, Locale::de];
    /// assert_eq!(Locale::en_GB.negotiate(&available), Some(Locale::en));
    /// ```
    pub fn negotiate(&self, available: &[Locale]) -> Option<Self> {{
        // First, try exact match
        if available.contains(self) {{
            return Some(*self);
        }}

        // Then, try fallback chain
        let mut current = *self;
        while let Some(fallback) = current.fallback() {{
            if available.contains(&fallback) {{
                return Some(fallback);
            }}
            current = fallback;
        }}

        None
    }}

    /// Suggests similar locales based on the input string.
    /// Returns up to 5 suggestions sorted by similarity.
    ///
    /// # Examples
    /// ```
    /// use locale_rs::Locale;
    ///
    /// let suggestions = Locale::suggest("en-gbb");
    /// assert!(suggestions.iter().any(|l| l.as_str() == "en-GB"));
    /// ```
    pub fn suggest(input: &str) -> Vec<Self> {{
        let normalized = input.replace('-', "_").to_lowercase();
        let mut suggestions = Vec::new();

        for &locale_str in AVAILABLE_LOCALES.iter() {{
            let locale_normalized = locale_str.replace('-', "_").to_lowercase();
            let distance = _levenshtein_distance(&normalized, &locale_normalized);

            // Only include if reasonably close (distance <= 3)
            if distance <= 3 {{
                suggestions.push((distance, locale_str));
            }}
        }}

        // Sort by distance, then by string for deterministic ordering
        suggestions.sort_by(|a, b| {{
            a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1))
        }});

        // Convert to Locale and limit to 5 results
        suggestions
            .into_iter()
            .take(5)
            .filter_map(|(_, s)| Self::from_str(s).ok())
            .collect()
    }}

    /// Returns the language code (primary subtag) of this locale.
    ///
    /// # Examples
    /// ```
    /// use locale_rs::Locale;
    ///
    /// assert_eq!(Locale::en_GB.language_code(), "en");
    /// assert_eq!(Locale::zh_Hans.language_code(), "zh");
    /// ```
    pub fn language_code(&self) -> &'static str {{
        match self {{
{lang_code_arms}        }}
    }}

    /// Returns the region code (territory subtag) of this locale, if present.
    ///
    /// # Examples
    /// ```
    /// use locale_rs::Locale;
    ///
    /// assert_eq!(Locale::en_GB.region_code(), Some("GB"));
    /// assert_eq!(Locale::en.region_code(), None);
    /// ```
    pub fn region_code(&self) -> Option<&'static str> {{
        match self {{
{region_code_arms}        }}
    }}
}}

/// Calculates the Levenshtein distance between two strings.
/// Used internally for locale suggestions.
fn _levenshtein_distance(s1: &str, s2: &str) -> usize {{
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {{
        return len2;
    }}
    if len2 == 0 {{
        return len1;
    }}

    let mut prev = vec![0; len2 + 1];
    let mut curr = vec![0; len2 + 1];

    for (i, prev_i) in prev.iter_mut().enumerate().take(len2 + 1) {{
        *prev_i = i;
    }}

    for (i, c1) in s1.chars().enumerate() {{
        curr[0] = i + 1;
        for (j, c2) in s2.chars().enumerate() {{
            let cost = if c1 == c2 {{ 0 }} else {{ 1 }};
            curr[j + 1] = std::cmp::min(
                std::cmp::min(curr[j] + 1, prev[j + 1] + 1),
                prev[j] + cost,
            );
        }}
        std::mem::swap(&mut prev, &mut curr);
    }}

    prev[len2]
}}

impl fmt::Display for Locale {{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{
        write!(f, "{{}}", self.as_str())
    }}
}}

impl FromStr for Locale {{
    type Err = LocaleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {{
        // Normalize input: replace underscores with hyphens and convert to lowercase
        let normalized = s.replace('_', "-").to_lowercase();
        // Now convert back to match AVAILABLE_LOCALES format by restoring proper case
        // We do this by finding the matching locale in AVAILABLE_LOCALES and using its case
        for &locale_str in AVAILABLE_LOCALES.iter() {{
            if locale_str.to_lowercase() == normalized {{
                return match locale_str {{
{from_str}                    _ => unreachable!(),
                }};
            }}
        }}
        Err(LocaleError::UnknownLocale(s.to_string()))
    }}
}}

impl TryFrom<&str> for Locale {{
    type Error = LocaleError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {{
        Self::from_str(value)
    }}
}}

impl From<Locale> for &'static str {{
    fn from(loc: Locale) -> Self {{
        loc.as_str()
    }}
}}

impl From<Locale> for String {{
    fn from(loc: Locale) -> Self {{
        loc.as_str().to_string()
    }}
}}

impl From<&Locale> for &'static str {{
    fn from(loc: &Locale) -> Self {{
        loc.as_str()
    }}
}}
"#,
        asset_name = asset_name,
        count = locales.len(),
        names = names,
        variants = variants,
        from_str = from_str,
        to_str = to_str,
        fallbacks = fallbacks,
        lang_code_arms = lang_code_arms,
        region_code_arms = region_code_arms
    );

    fs::write(output_path, code)?;
    tracing::info!("Generated {} locales.", locales.len());
    Ok(())
}
