//! The [`Locale`] identifier and locale negotiation.

use crate::data::locales::{LOCALE_MAP, PARENTS};
use crate::error::LocaleError;
use std::fmt;
use std::str::FromStr;

pub use crate::data::locales::{AVAILABLE_LOCALES, CLDR_VERSION, Locale};

impl Locale {
    /// Position of this locale in [`AVAILABLE_LOCALES`] and in every data table.
    #[inline]
    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    /// Returns the BCP 47 identifier of the locale, e.g. `"en-GB"`.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        AVAILABLE_LOCALES[self.index()]
    }

    /// Returns the locale this one inherits its data from, following the CLDR
    /// parent locale rules.
    ///
    /// Usually this strips the last subtag, but CLDR overrides it where the
    /// data differs: `en-IN` inherits from `en-001` (international English),
    /// and a locale in a script that is not the language's default, such as
    /// `az-Cyrl`, inherits from nothing.
    ///
    /// ```
    /// use locale_rs::Locale;
    ///
    /// assert_eq!(Locale::de_AT.fallback(), Some(Locale::de));
    /// assert_eq!(Locale::en_IN.fallback(), Some(Locale::en_001));
    /// assert_eq!(Locale::en_001.fallback(), Some(Locale::en));
    /// assert_eq!(Locale::en.fallback(), None);
    /// ```
    #[inline]
    pub fn fallback(self) -> Option<Self> {
        PARENTS[self.index()]
    }

    /// Iterates over this locale and all of its [`fallback`](Self::fallback)s.
    ///
    /// ```
    /// use locale_rs::Locale;
    ///
    /// let chain: Vec<_> = Locale::en_IN.fallback_chain().collect();
    /// assert_eq!(chain, [Locale::en_IN, Locale::en_001, Locale::en]);
    /// ```
    pub fn fallback_chain(self) -> impl Iterator<Item = Self> {
        std::iter::successors(Some(self), |l| l.fallback())
    }

    /// Parses a locale string with flexible formatting.
    /// Accepts both hyphens and underscores, and is case-insensitive.
    ///
    /// This is the same as [`FromStr`], which is equally lenient.
    ///
    /// ```
    /// use locale_rs::Locale;
    ///
    /// assert_eq!(Locale::from_flexible("en-GB"), Ok(Locale::en_GB));
    /// assert_eq!(Locale::from_flexible("en_gb"), Ok(Locale::en_GB));
    /// assert_eq!(Locale::from_flexible("EN-gb"), Ok(Locale::en_GB));
    /// ```
    pub fn from_flexible(s: &str) -> Result<Self, LocaleError> {
        Self::from_str(s)
    }

    /// Finds the best matching locale from a list of available locales by
    /// walking this locale's [`fallback_chain`](Self::fallback_chain).
    ///
    /// ```
    /// use locale_rs::Locale;
    ///
    /// let available = vec![Locale::en, Locale::de];
    /// assert_eq!(Locale::en_GB.negotiate(&available), Some(Locale::en));
    /// ```
    pub fn negotiate(self, available: &[Locale]) -> Option<Self> {
        self.fallback_chain().find(|l| available.contains(l))
    }

    /// Suggests similar locales based on the input string.
    /// Returns up to 5 suggestions sorted by similarity.
    ///
    /// ```
    /// use locale_rs::Locale;
    ///
    /// let suggestions = Locale::suggest("en-gbb");
    /// assert!(suggestions.iter().any(|l| l.as_str() == "en-GB"));
    /// ```
    pub fn suggest(input: &str) -> Vec<Self> {
        let normalized = normalize(input);
        let mut suggestions: Vec<(usize, Locale)> = LOCALE_MAP
            .entries()
            .filter_map(|(key, &locale)| {
                let distance = levenshtein_distance(&normalized, key);
                (distance <= 3).then_some((distance, locale))
            })
            .collect();

        // Sort by distance, then by identifier for deterministic ordering.
        suggestions.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.as_str().cmp(b.1.as_str())));
        suggestions.into_iter().take(5).map(|(_, l)| l).collect()
    }

    /// Returns the language code (primary subtag) of this locale.
    ///
    /// ```
    /// use locale_rs::Locale;
    ///
    /// assert_eq!(Locale::en_GB.language_code(), "en");
    /// assert_eq!(Locale::zh_Hans.language_code(), "zh");
    /// ```
    pub fn language_code(self) -> &'static str {
        let id = self.as_str();
        id.split('-').next().unwrap_or(id)
    }

    /// Returns the region subtag of this locale, if present: either two
    /// letters (`GB`) or a three-digit UN M.49 area code (`001`).
    ///
    /// ```
    /// use locale_rs::Locale;
    ///
    /// assert_eq!(Locale::en_GB.region_code(), Some("GB"));
    /// assert_eq!(Locale::en_001.region_code(), Some("001"));
    /// assert_eq!(Locale::ca_ES_valencia.region_code(), Some("ES"));
    /// assert_eq!(Locale::zh_Hans.region_code(), None);
    /// ```
    pub fn region_code(self) -> Option<&'static str> {
        self.as_str().split('-').skip(1).find(|tag| is_region(tag))
    }
}

fn is_region(tag: &str) -> bool {
    match tag.len() {
        2 => tag.bytes().all(|b| b.is_ascii_uppercase()),
        3 => tag.bytes().all(|b| b.is_ascii_digit()),
        _ => false,
    }
}

/// Lowercases and replaces underscores, the key format of `LOCALE_MAP`.
fn normalize(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c == '_' {
                '-'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect()
}

/// Levenshtein distance over characters. Used for locale suggestions.
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s2: Vec<char> = s2.chars().collect();
    let mut prev: Vec<usize> = (0..=s2.len()).collect();
    let mut curr = vec![0; s2.len() + 1];

    for (i, c1) in s1.chars().enumerate() {
        curr[0] = i + 1;
        for (j, &c2) in s2.iter().enumerate() {
            let cost = usize::from(c1 != c2);
            curr[j + 1] = (curr[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[s2.len()]
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

impl FromStr for Locale {
    type Err = LocaleError;

    /// Parses a locale identifier. Underscores are accepted in place of
    /// hyphens and case is ignored.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LOCALE_MAP
            .get(normalize(s).as_str())
            .copied()
            .ok_or_else(|| LocaleError::UnknownLocale(s.to_string()))
    }
}

impl TryFrom<&str> for Locale {
    type Error = LocaleError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl From<Locale> for &'static str {
    fn from(loc: Locale) -> Self {
        loc.as_str()
    }
}

impl From<Locale> for String {
    fn from(loc: Locale) -> Self {
        loc.as_str().to_string()
    }
}

impl From<&Locale> for &'static str {
    fn from(loc: &Locale) -> Self {
        loc.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_counts_characters_not_bytes() {
        assert_eq!(levenshtein_distance("ä", "a"), 1);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn region_subtags() {
        assert!(is_region("DE"));
        assert!(is_region("419"));
        assert!(!is_region("Latn"));
        assert!(!is_region("valencia"));
        assert!(!is_region("de"));
    }
}
