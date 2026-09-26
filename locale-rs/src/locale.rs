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
        let input: Vec<char> = normalize(input).chars().collect();
        let counts = CharCounts::new(&input);
        let mut suggestions: Vec<(usize, Locale)> = LOCALE_MAP
            .entries()
            .filter_map(|(key, &locale)| {
                bounded_levenshtein(&input, &counts, key, MAX_SUGGESTION_DISTANCE)
                    .map(|d| (d, locale))
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

/// Largest edit distance `suggest` reports.
const MAX_SUGGESTION_DISTANCE: usize = 3;

/// Upper bound on the length of a locale identifier, checked by a test.
const MAX_ID_LEN: usize = 32;

/// Counts of the characters of a `suggest` input: one slot per ASCII byte,
/// plus the number of non-ASCII characters, which match no identifier.
struct CharCounts {
    ascii: [u8; 128],
    non_ascii: usize,
}

impl CharCounts {
    fn new(chars: &[char]) -> Self {
        let mut counts = Self {
            ascii: [0; 128],
            non_ascii: 0,
        };
        for &c in chars {
            match u8::try_from(c) {
                Ok(b) if b < 128 => {
                    counts.ascii[usize::from(b)] = counts.ascii[usize::from(b)].saturating_add(1)
                }
                _ => counts.non_ascii += 1,
            }
        }
        counts
    }

    /// A lower bound on the edit distance to the ASCII string `b`: every
    /// character of one string without a partner in the other needs an edit.
    fn lower_bound(&self, len: usize, b: &[u8]) -> usize {
        let mut left = self.ascii;
        let mut unmatched_b = 0;
        for &byte in b {
            match left.get_mut(usize::from(byte)) {
                Some(n) if *n > 0 => *n -= 1,
                _ => unmatched_b += 1,
            }
        }
        let unmatched_a = len - (b.len() - unmatched_b);
        unmatched_a.max(unmatched_b)
    }
}

/// Levenshtein distance between `a` and the ASCII identifier `b`, or `None`
/// if it exceeds `max`.
///
/// Cheap lower bounds (length difference, unmatched characters) rule out
/// most identifiers before the dynamic program runs. The program itself uses
/// small stack rows and stops as soon as every entry of a row exceeds `max`
/// (row minima never decrease).
fn bounded_levenshtein(a: &[char], counts: &CharCounts, b: &str, max: usize) -> Option<usize> {
    let b = b.as_bytes();
    if a.len().abs_diff(b.len()) > max || b.len() > MAX_ID_LEN {
        return None;
    }
    if counts.lower_bound(a.len(), b) > max {
        return None;
    }

    // Distances above `max` are clamped, so `u8` cells cannot overflow.
    let cap = u8::try_from(max + 1).unwrap_or(u8::MAX);
    let mut prev = [0u8; MAX_ID_LEN + 1];
    let mut curr = [0u8; MAX_ID_LEN + 1];
    for (j, p) in prev.iter_mut().enumerate().take(b.len() + 1) {
        *p = u8::try_from(j).unwrap_or(u8::MAX).min(cap);
    }

    for (i, &ca) in a.iter().enumerate() {
        curr[0] = u8::try_from(i + 1).unwrap_or(u8::MAX).min(cap);
        let mut row_min = curr[0];
        for (j, &cb) in b.iter().enumerate() {
            let cost = u8::from(ca != char::from(cb));
            curr[j + 1] = (curr[j] + 1)
                .min(prev[j + 1] + 1)
                .min(prev[j] + cost)
                .min(cap);
            row_min = row_min.min(curr[j + 1]);
        }
        if row_min >= cap {
            return None;
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    let distance = usize::from(prev[b.len()]);
    (distance <= max).then_some(distance)
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

    /// Plain Levenshtein distance over characters, the reference for
    /// `bounded_levenshtein`.
    fn levenshtein(a: &str, b: &str) -> usize {
        let b: Vec<char> = b.chars().collect();
        let mut prev: Vec<usize> = (0..=b.len()).collect();
        for (i, ca) in a.chars().enumerate() {
            let mut curr = vec![i + 1; b.len() + 1];
            for (j, &cb) in b.iter().enumerate() {
                curr[j + 1] = (curr[j] + 1)
                    .min(prev[j + 1] + 1)
                    .min(prev[j] + usize::from(ca != cb));
            }
            prev = curr;
        }
        prev[b.len()]
    }

    fn bounded(a: &str, b: &str) -> Option<usize> {
        let a: Vec<char> = a.chars().collect();
        bounded_levenshtein(&a, &CharCounts::new(&a), b, MAX_SUGGESTION_DISTANCE)
    }

    #[test]
    fn bounded_levenshtein_basics() {
        assert_eq!(bounded("ä", "a"), Some(1));
        assert_eq!(bounded("", "abc"), Some(3));
        assert_eq!(bounded("", "abcd"), None);
        assert_eq!(bounded("kitten", "sitting"), Some(3));
        assert_eq!(bounded("en-gb", "en-gb"), Some(0));
        assert_eq!(bounded("zzzzzz", "en-gb"), None);
    }

    #[test]
    fn bounded_levenshtein_matches_reference() {
        let keys: Vec<String> = AVAILABLE_LOCALES.iter().map(|id| normalize(id)).collect();
        let mut inputs = vec![
            String::new(),
            "x".into(),
            "en-gbb".into(),
            "pt_br".into(),
            "zh-hant-hkk".into(),
            "ääää".into(),
            "sr-latn-ba-extra".into(),
        ];
        // Deletions, insertions and substitutions of real identifiers.
        for key in keys.iter().step_by(7) {
            for i in 0..key.len() {
                inputs.push(format!("{}{}", &key[..i], &key[i + 1..]));
                inputs.push(format!("{}q{}", &key[..i], &key[i..]));
                inputs.push(format!("{}ü{}", &key[..i], &key[i + 1..]));
            }
        }
        for input in &inputs {
            for key in keys.iter().step_by(3) {
                let d = levenshtein(input, key);
                let expected = (d <= MAX_SUGGESTION_DISTANCE).then_some(d);
                assert_eq!(bounded(input, key), expected, "{input:?} vs {key:?}");
            }
        }
    }

    #[test]
    fn identifiers_fit_the_suggestion_rows() {
        for id in AVAILABLE_LOCALES {
            assert!(id.is_ascii() && id.len() <= MAX_ID_LEN, "{id}");
        }
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
