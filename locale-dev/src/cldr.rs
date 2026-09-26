//! Reads a CLDR JSON archive once into a typed model.
//!
//! The emitters in [`crate::emit`] only ever see this model, never JSON, so
//! all knowledge of the CLDR file layout lives in this module.

use crate::error::{Error, Result};
use rayon::prelude::*;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// Everything `locale-rs` needs from one CLDR release.
#[derive(Debug, Clone, PartialEq)]
pub struct Cldr {
    /// All locales, sorted by identifier. Indices into this list are the
    /// discriminants of the generated `Locale` enum.
    pub locales: Vec<LocaleData>,
    /// Fraction digits of every currency that does not use 2.
    pub fraction_digits: BTreeMap<String, u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleData {
    /// BCP 47 identifier, e.g. `en-GB`.
    pub name: String,
    /// Index of the locale this one inherits from.
    pub parent: Option<usize>,
    pub numbers: NumberData,
    pub dates: DateData,
    /// Standard currency pattern, e.g. `¤#,##0.00`.
    pub currency_pattern: String,
    /// ISO code of the currency of the locale's region.
    pub default_currency: String,
    /// Currency code to symbol, for every currency the locale names.
    pub currency_symbols: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberData {
    pub decimal: String,
    pub group: String,
    pub minus_sign: String,
    /// Standard decimal pattern, e.g. `#,##0.###`.
    pub decimal_pattern: String,
    /// Native digits, unless the default numbering system is `latn`.
    pub digits: Option<[char; 10]>,
}

impl Default for NumberData {
    fn default() -> Self {
        Self {
            decimal: ".".into(),
            group: ",".into(),
            minus_sign: "-".into(),
            decimal_pattern: "#,##0.###".into(),
            digits: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateData {
    pub months_wide: [String; 12],
    pub months_abbreviated: [String; 12],
    /// Sunday first.
    pub weekdays_wide: [String; 7],
    pub am: String,
    pub pm: String,
    pub date_pattern: String,
    pub time_pattern: String,
}

impl Default for DateData {
    fn default() -> Self {
        let months: [String; 12] = std::array::from_fn(|i| (i + 1).to_string());
        Self {
            months_wide: months.clone(),
            months_abbreviated: months,
            weekdays_wide: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].map(String::from),
            am: "AM".into(),
            pm: "PM".into(),
            date_pattern: "y-MM-dd".into(),
            time_pattern: "HH:mm:ss".into(),
        }
    }
}

const DEFAULT_CURRENCY_PATTERN: &str = "¤#,##0.00";

/// A zip archive with the layout of the `cldr-json` release assets
/// (`cldr-core/`, `cldr-numbers-full/`, `cldr-dates-full/`, ...).
///
/// Cloning is cheap: clones share the buffer and the parsed central
/// directory, so each worker thread reads through its own handle.
#[derive(Clone)]
struct Archive<'a> {
    zip: ZipArchive<Cursor<&'a [u8]>>,
}

impl Archive<'_> {
    fn json(&mut self, path: &str) -> Result<Option<Value>> {
        let mut file = match self.zip.by_name(path) {
            Ok(file) => file,
            Err(zip::result::ZipError::FileNotFound) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let mut text = String::new();
        file.read_to_string(&mut text)?;
        serde_json::from_str(&text)
            .map(Some)
            .map_err(|source| Error::Json {
                path: path.to_string(),
                source,
            })
    }

    fn required_json(&mut self, path: &str) -> Result<Value> {
        self.json(path)?
            .ok_or_else(|| Error::Cldr(format!("missing {path}")))
    }

    /// Names of all locales: the directories below any `*/main/`.
    fn locale_names(&self) -> Vec<String> {
        let names: BTreeSet<String> = self
            .zip
            .file_names()
            .filter_map(|path| {
                let mut parts = path.split('/');
                parts.find(|&p| p == "main")?;
                let name = parts.next()?;
                // Only directories count, never a stray file in `main/`.
                (!name.is_empty() && parts.next().is_some()).then(|| name.to_string())
            })
            .collect();
        names.into_iter().collect()
    }
}

impl Cldr {
    /// Parses a CLDR JSON archive. Locales are read in parallel.
    pub fn from_zip(buffer: Vec<u8>) -> Result<Self> {
        let mut archive = Archive {
            zip: ZipArchive::new(Cursor::new(buffer.as_slice()))?,
        };
        let names = archive.locale_names();
        if names.is_empty() {
            return Err(Error::Cldr("the archive contains no locales".into()));
        }

        let numbering_systems = numbering_systems(&mut archive)?;
        let likely = likely_subtags(&mut archive)?;
        let parent_overrides = parent_locales(&mut archive)?;
        let (region_currency, fraction_digits) = currency_data(&mut archive)?;

        let index: HashMap<&str, usize> = names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.as_str(), i))
            .collect();

        let read_locale = |archive: &mut Archive, name: &String| -> Result<LocaleData> {
            let numbers_json =
                archive.json(&format!("cldr-numbers-full/main/{name}/numbers.json"))?;
            let numbers_json = numbers_json.as_ref().map(|j| &j["main"][name]["numbers"]);
            let (numbers, currency_pattern) = number_data(numbers_json, &numbering_systems);

            let currencies =
                archive.json(&format!("cldr-numbers-full/main/{name}/currencies.json"))?;
            let currency_symbols = currencies
                .as_ref()
                .and_then(|j| j["main"][name]["numbers"]["currencies"].as_object())
                .map(|currencies| {
                    currencies
                        .iter()
                        .filter(|(code, _)| is_currency_code(code))
                        .map(|(code, data)| {
                            let symbol = data["symbol"].as_str().unwrap_or(code);
                            (code.clone(), symbol.to_string())
                        })
                        .collect()
                })
                .unwrap_or_default();

            let gregorian =
                archive.json(&format!("cldr-dates-full/main/{name}/ca-gregorian.json"))?;
            let dates = gregorian
                .as_ref()
                .map(|j| date_data(&j["main"][name]["dates"]["calendars"]["gregorian"]))
                .unwrap_or_default();

            Ok(LocaleData {
                name: name.clone(),
                parent: resolve_parent(name, &parent_overrides, &likely, &index),
                numbers,
                dates,
                currency_pattern,
                default_currency: default_currency(name, &likely, &region_currency),
                currency_symbols,
            })
        };
        // Every task gets its own archive handle; `collect` keeps the order.
        let locales = names
            .par_iter()
            .map_init(|| archive.clone(), read_locale)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            locales,
            fraction_digits,
        })
    }

    /// Follows the parent chain of the locale at `index`, starting with itself.
    pub fn chain(&self, index: usize) -> impl Iterator<Item = &LocaleData> {
        std::iter::successors(Some(index), |&i| self.locales[i].parent).map(|i| &self.locales[i])
    }
}

pub fn is_currency_code(code: &str) -> bool {
    code.len() == 3 && code.bytes().all(|b| b.is_ascii_uppercase())
}

fn numbering_systems(archive: &mut Archive) -> Result<HashMap<String, [char; 10]>> {
    let json = archive.required_json("cldr-core/supplemental/numberingSystems.json")?;
    let mut systems = HashMap::new();
    if let Some(all) = json["supplemental"]["numberingSystems"].as_object() {
        for (name, data) in all {
            if data["_type"].as_str() != Some("numeric") {
                continue;
            }
            let digits: Vec<char> = data["_digits"].as_str().unwrap_or("").chars().collect();
            if let Ok(digits) = <[char; 10]>::try_from(digits) {
                systems.insert(name.clone(), digits);
            }
        }
    }
    Ok(systems)
}

/// Maps a language (or language-script, ...) to its likely full identifier,
/// e.g. `de` to `de-Latn-DE`.
fn likely_subtags(archive: &mut Archive) -> Result<HashMap<String, String>> {
    let json = archive.required_json("cldr-core/supplemental/likelySubtags.json")?;
    Ok(json["supplemental"]["likelySubtags"]
        .as_object()
        .map(|all| {
            all.iter()
                .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default())
}

/// Explicit parent locales, e.g. `en-IN` to `en-001`.
fn parent_locales(archive: &mut Archive) -> Result<HashMap<String, String>> {
    let json = archive.required_json("cldr-core/supplemental/parentLocales.json")?;
    Ok(json["supplemental"]["parentLocales"]["parentLocale"]
        .as_object()
        .map(|all| {
            all.iter()
                .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default())
}

type CurrencyTables = (HashMap<String, String>, BTreeMap<String, u8>);

/// The current legal tender of every region, and non-default fraction digits.
fn currency_data(archive: &mut Archive) -> Result<CurrencyTables> {
    let json = archive.required_json("cldr-core/supplemental/currencyData.json")?;
    let data = &json["supplemental"]["currencyData"];

    let mut regions = HashMap::new();
    if let Some(all) = data["region"].as_object() {
        for (region, history) in all {
            let current = history.as_array().into_iter().flatten().find_map(|entry| {
                let (code, info) = entry.as_object()?.iter().next()?;
                let current = info["_to"].is_null() && info["_tender"].as_str() != Some("false");
                current.then(|| code.clone())
            });
            if let Some(code) = current {
                regions.insert(region.clone(), code);
            }
        }
    }

    let mut fractions = BTreeMap::new();
    if let Some(all) = data["fractions"].as_object() {
        for (code, info) in all {
            let digits = info["_digits"].as_str().and_then(|d| d.parse().ok());
            if let Some(digits) = digits.filter(|&d| d != 2)
                && is_currency_code(code)
            {
                fractions.insert(code.clone(), digits);
            }
        }
    }

    Ok((regions, fractions))
}

fn number_data(
    numbers: Option<&Value>,
    systems: &HashMap<String, [char; 10]>,
) -> (NumberData, String) {
    let mut data = NumberData::default();
    let mut currency_pattern = DEFAULT_CURRENCY_PATTERN.to_string();
    let Some(numbers) = numbers else {
        return (data, currency_pattern);
    };

    let system = numbers["defaultNumberingSystem"].as_str().unwrap_or("latn");
    let symbols = &numbers[format!("symbols-numberSystem-{system}")];
    let text = |v: &Value, target: &mut String| {
        if let Some(s) = v.as_str() {
            *target = s.to_string();
        }
    };
    text(&symbols["decimal"], &mut data.decimal);
    text(&symbols["group"], &mut data.group);
    text(&symbols["minusSign"], &mut data.minus_sign);
    text(
        &numbers[format!("decimalFormats-numberSystem-{system}")]["standard"],
        &mut data.decimal_pattern,
    );
    text(
        &numbers[format!("currencyFormats-numberSystem-{system}")]["standard"],
        &mut currency_pattern,
    );
    if system != "latn" {
        data.digits = systems.get(system).copied();
    }
    (data, currency_pattern)
}

fn date_data(gregorian: &Value) -> DateData {
    let defaults = DateData::default();
    let text = |v: &Value, default: &str| v.as_str().unwrap_or(default).to_string();
    DateData {
        months_wide: months(&gregorian["months"]["format"]["wide"]),
        months_abbreviated: months(&gregorian["months"]["format"]["abbreviated"]),
        weekdays_wide: weekdays(&gregorian["days"]["format"]["wide"]),
        am: text(
            &gregorian["dayPeriods"]["format"]["wide"]["am"],
            &defaults.am,
        ),
        pm: text(
            &gregorian["dayPeriods"]["format"]["wide"]["pm"],
            &defaults.pm,
        ),
        date_pattern: text(&gregorian["dateFormats"]["medium"], &defaults.date_pattern),
        time_pattern: text(&gregorian["timeFormats"]["medium"], &defaults.time_pattern),
    }
}

/// Month names keyed `"1"` to `"12"`; missing names become empty strings.
pub fn months(obj: &Value) -> [String; 12] {
    std::array::from_fn(|i| obj[(i + 1).to_string()].as_str().unwrap_or("").to_string())
}

/// Weekday names keyed `"sun"` to `"sat"`; missing names become empty strings.
pub fn weekdays(obj: &Value) -> [String; 7] {
    ["sun", "mon", "tue", "wed", "thu", "fri", "sat"]
        .map(|key| obj[key].as_str().unwrap_or("").to_string())
}

fn is_script(tag: &str) -> bool {
    tag.len() == 4 && tag.bytes().all(|b| b.is_ascii_alphabetic())
}

/// The CLDR parent of a locale identifier, which need not be an available
/// locale. `None` means root.
pub fn parent_name(
    name: &str,
    overrides: &HashMap<String, String>,
    likely: &HashMap<String, String>,
) -> Option<String> {
    if let Some(parent) = overrides.get(name) {
        return (parent != "root" && parent != "und").then(|| parent.clone());
    }
    let (truncated, last) = name.rsplit_once('-')?;
    // `nonlikelyScript`: a language-script locale whose script is not the
    // language's likely one inherits from root, not from the language.
    if is_script(last) && !truncated.contains('-') {
        let likely_script = likely
            .get(truncated)
            .and_then(|full| full.split('-').nth(1));
        if likely_script != Some(last) {
            return None;
        }
    }
    Some(truncated.to_string())
}

/// The nearest ancestor of `name` that is an available locale.
pub fn resolve_parent(
    name: &str,
    overrides: &HashMap<String, String>,
    likely: &HashMap<String, String>,
    available: &HashMap<&str, usize>,
) -> Option<usize> {
    let mut current = parent_name(name, overrides, likely)?;
    loop {
        if let Some(&i) = available.get(current.as_str()) {
            return Some(i);
        }
        current = parent_name(&current, overrides, likely)?;
    }
}

fn region_of(name: &str) -> Option<&str> {
    name.split('-').skip(1).find(|t| {
        (t.len() == 2 && t.bytes().all(|b| b.is_ascii_uppercase()))
            || (t.len() == 3 && t.bytes().all(|b| b.is_ascii_digit()))
    })
}

/// The currency of the locale's country. A locale without a region uses the
/// likely country of its language (`de` is `DE`); a macro-region such as
/// `419` or `150` has no currency of its own and gets `USD`.
pub fn default_currency(
    name: &str,
    likely: &HashMap<String, String>,
    region_currency: &HashMap<String, String>,
) -> String {
    let region = match region_of(name) {
        Some(region) if region.bytes().all(|b| b.is_ascii_digit()) => None,
        Some(region) => Some(region.to_string()),
        None => {
            // Most specific likely-subtags entry: `sr-Latn`, then `sr`.
            let mut candidate = name;
            loop {
                if let Some(region) = likely.get(candidate).and_then(|full| region_of(full)) {
                    break Some(region.to_string());
                }
                match candidate.rsplit_once('-') {
                    Some((head, _)) => candidate = head,
                    None => break None,
                }
            }
        }
    };
    region
        .and_then(|r| region_currency.get(&r).cloned())
        .unwrap_or_else(|| "USD".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn months_are_ordered_and_gaps_are_empty() {
        let m = months(&json!({ "1": "Jan", "12": "Dec" }));
        assert_eq!(m[0], "Jan");
        assert_eq!(m[5], "");
        assert_eq!(m[11], "Dec");
        assert!(months(&json!("not an object")).iter().all(String::is_empty));
    }

    #[test]
    fn weekdays_are_sunday_first() {
        let v = json!({
            "sun": "Su", "mon": "Mo", "tue": "Tu", "wed": "We",
            "thu": "Th", "fri": "Fr", "sat": "Sa"
        });
        assert_eq!(weekdays(&v), ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"]);
        assert_eq!(weekdays(&json!({ "mon": "Mo" }))[0], "");
    }

    #[test]
    fn parents_follow_overrides_truncation_and_script_rule() {
        let overrides = map(&[("en-IN", "en-001"), ("az-Arab", "und")]);
        let likely = map(&[("sr", "sr-Cyrl-RS"), ("zh", "zh-Hans-CN")]);
        let p = |n| parent_name(n, &overrides, &likely);
        assert_eq!(p("en-IN").as_deref(), Some("en-001"));
        assert_eq!(p("en-001").as_deref(), Some("en"));
        assert_eq!(p("az-Arab"), None);
        assert_eq!(p("sr-Latn"), None);
        assert_eq!(p("sr-Cyrl").as_deref(), Some("sr"));
        assert_eq!(p("sr-Latn-BA").as_deref(), Some("sr-Latn"));
        assert_eq!(p("zh-Hant-HK").as_deref(), Some("zh-Hant"));
        assert_eq!(p("de"), None);
    }

    #[test]
    fn parents_skip_unavailable_ancestors() {
        let available: HashMap<&str, usize> = [("de", 0), ("de-AT-x", 1)].into_iter().collect();
        let none = HashMap::new();
        assert_eq!(resolve_parent("de-AT-x", &none, &none, &available), Some(0));
        assert_eq!(resolve_parent("de", &none, &none, &available), None);
    }

    #[test]
    fn default_currency_prefers_own_region() {
        let likely = map(&[
            ("de", "de-Latn-DE"),
            ("en", "en-Latn-US"),
            ("es", "es-Latn-ES"),
        ]);
        let regions = map(&[
            ("DE", "EUR"),
            ("AT", "EUR"),
            ("CH", "CHF"),
            ("US", "USD"),
            ("ES", "EUR"),
        ]);
        let c = |n| default_currency(n, &likely, &regions);
        assert_eq!(c("de"), "EUR");
        assert_eq!(c("de-CH"), "CHF");
        assert_eq!(c("en"), "USD");
        assert_eq!(c("en-001"), "USD");
        assert_eq!(c("es-419"), "USD");
        assert_eq!(c("xx"), "USD");
    }
}
