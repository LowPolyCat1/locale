//! Locale-aware currency formatting.
//!
//! A [`CurrencyFormatter`] combines a locale's currency pattern and number
//! symbols with a [`Currency`]. Unless another currency is given, it uses the
//! locale's default currency: the one in circulation in the locale's region.
//!
//! ```
//! use locale_rs::Locale;
//! use locale_rs::currency::{Currency, CurrencyFormatter, ToCurrencyString};
//!
//! assert_eq!(1234.5.to_currency(&Locale::en), "$1,234.50");
//! assert_eq!(1234.5.to_currency(&Locale::de_AT), "€\u{a0}1\u{a0}234,50");
//!
//! let yen: Currency = "JPY".parse().unwrap();
//! assert_eq!(1234.5.to_currency_in(&Locale::en, yen), "¥1,234");
//!
//! let chf = CurrencyFormatter::with_currency(Locale::de_CH, "CHF".parse().unwrap());
//! assert_eq!(chf.format(-5).to_string(), "CHF-5.00");
//! ```

use crate::Locale;
use crate::data::currency::{
    CURRENCY_PATTERNS, CURRENCY_SYMBOLS, DEFAULT_CURRENCIES, FRACTION_DIGITS,
};
use crate::error::LocaleError;
use crate::format::{
    self as fmt_util, FloatStr, WriteTo, u128_digits, write_digits, write_grouped,
};
use crate::nums::{Float, Number, NumberSymbols, NumberValue};
use std::fmt::{self, Write};
use std::str::FromStr;

pub use crate::data::Grouping;

/// An ISO 4217 currency code such as `EUR`.
///
/// Any three ASCII letters are accepted; codes unknown to CLDR are formatted
/// with the code as symbol and two fraction digits.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Currency(pub(crate) [u8; 3]);

impl Currency {
    /// Parses a three-letter currency code, ignoring case.
    ///
    /// ```
    /// use locale_rs::currency::Currency;
    ///
    /// assert_eq!(Currency::new("eur").unwrap().as_str(), "EUR");
    /// assert!(Currency::new("EURO").is_err());
    /// ```
    pub fn new(code: &str) -> Result<Self, LocaleError> {
        match code.as_bytes() {
            &[a, b, c] if [a, b, c].iter().all(u8::is_ascii_alphabetic) => Ok(Self([
                a.to_ascii_uppercase(),
                b.to_ascii_uppercase(),
                c.to_ascii_uppercase(),
            ])),
            _ => Err(LocaleError::InvalidCurrency(code.to_string())),
        }
    }

    /// The uppercase currency code.
    pub fn as_str(&self) -> &str {
        // Construction guarantees three ASCII letters.
        std::str::from_utf8(&self.0).unwrap_or_default()
    }

    /// The currency in circulation in the country of `locale`, e.g. `EUR`
    /// for `de-AT` and `CHF` for `de-CH`.
    ///
    /// A locale without a region subtag uses its language's most likely
    /// country, so `de` gets `EUR` and `en` gets `USD`. Macro-regions such as
    /// `es-419` have no currency of their own and get `USD`; pass a currency
    /// explicitly where that matters.
    ///
    /// ```
    /// use locale_rs::{Locale, currency::Currency};
    ///
    /// assert_eq!(Currency::default_for(Locale::de_AT).as_str(), "EUR");
    /// assert_eq!(Currency::default_for(Locale::sr_Latn).as_str(), "RSD");
    /// ```
    pub fn default_for(locale: Locale) -> Self {
        DEFAULT_CURRENCIES[locale.index()]
    }

    /// Number of fraction digits the currency is normally written with,
    /// e.g. 2 for `EUR`, 0 for `JPY` and 3 for `KWD`.
    pub fn fraction_digits(self) -> u8 {
        FRACTION_DIGITS
            .binary_search_by_key(&self, |&(c, _)| c)
            .map_or(2, |i| FRACTION_DIGITS[i].1)
    }
}

impl fmt::Debug for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Currency").field(&self.as_str()).finish()
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

impl FromStr for Currency {
    type Err = LocaleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// A CLDR currency pattern, split around its number.
///
/// `¤` in an affix stands for the currency symbol. Negative affixes already
/// contain the locale's minus sign.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CurrencyPattern {
    pub(crate) positive_prefix: &'static str,
    pub(crate) positive_suffix: &'static str,
    pub(crate) negative_prefix: &'static str,
    pub(crate) negative_suffix: &'static str,
    pub(crate) grouping: Grouping,
}

/// Looks up the symbol of `currency` along the fallback chain of `locale`.
fn symbol_for(locale: Locale, currency: Currency) -> Option<&'static str> {
    locale.fallback_chain().find_map(|l| {
        let table = CURRENCY_SYMBOLS[l.index()];
        table
            .binary_search_by_key(&currency, |&(c, _)| c)
            .ok()
            .map(|i| table[i].1)
    })
}

/// Formats amounts of one currency for one locale.
#[derive(Debug, Clone, Copy)]
pub struct CurrencyFormatter {
    numbers: &'static NumberSymbols,
    pattern: &'static CurrencyPattern,
    currency: Currency,
    symbol: Option<&'static str>,
}

impl CurrencyFormatter {
    /// Creates a formatter for the [default currency](Currency::default_for)
    /// of `locale`.
    pub fn new(locale: Locale) -> Self {
        Self::with_currency(locale, Currency::default_for(locale))
    }

    /// Creates a formatter for `currency` in `locale`.
    pub fn with_currency(locale: Locale, currency: Currency) -> Self {
        Self {
            numbers: NumberSymbols::for_locale(locale),
            pattern: CURRENCY_PATTERNS[locale.index()],
            currency,
            symbol: symbol_for(locale, currency),
        }
    }

    /// The currency this formatter writes.
    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// The localized symbol of the currency, e.g. `€` or `US$`. Falls back to
    /// the currency code.
    pub fn symbol(&self) -> &str {
        match self.symbol {
            Some(s) => s,
            None => self.currency.as_str(),
        }
    }

    /// Returns `value` wrapped for display.
    pub fn format<N: Number>(&self, value: N) -> FormattedCurrency<N> {
        FormattedCurrency {
            value,
            formatter: *self,
        }
    }

    fn write_affix<W: Write + ?Sized>(&self, out: &mut W, affix: &str) -> fmt::Result {
        let mut parts = affix.split('¤');
        if let Some(first) = parts.next() {
            out.write_str(first)?;
        }
        for part in parts {
            out.write_str(self.symbol())?;
            out.write_str(part)?;
        }
        Ok(())
    }

    fn write<W: Write + ?Sized>(&self, out: &mut W, value: NumberValue) -> fmt::Result {
        let fraction = usize::from(self.currency.fraction_digits());
        let mut int_buf = [0; 40];
        let mut float_buf = FloatStr::new();

        // The unsigned amount as ASCII: integer digits and fraction digits.
        let (negative, int, frac): (bool, &str, &str) = match value {
            NumberValue::Int { negative, abs } => (negative, u128_digits(&mut int_buf, abs), ""),
            NumberValue::F32(v) => {
                let negative = round_abs(&mut float_buf, v, fraction)?;
                let (int, frac) = split_decimal(float_buf.as_str());
                (negative, int, frac)
            }
            NumberValue::F64(v) => {
                let negative = round_abs(&mut float_buf, v, fraction)?;
                let (int, frac) = split_decimal(float_buf.as_str());
                (negative, int, frac)
            }
        };

        let p = self.pattern;
        let (prefix, suffix) = if negative {
            (p.negative_prefix, p.negative_suffix)
        } else {
            (p.positive_prefix, p.positive_suffix)
        };
        let n = self.numbers;

        self.write_affix(out, prefix)?;
        if !int.bytes().all(|b| b.is_ascii_digit()) {
            // NaN or infinity.
            out.write_str(int)?;
        } else {
            write_grouped(out, int, p.grouping, n.group, n.digits)?;
            if fraction > 0 {
                out.write_str(n.decimal)?;
                write_digits(out, frac, n.digits)?;
                for _ in frac.len()..fraction {
                    write_digits(out, "0", n.digits)?;
                }
            }
        }
        self.write_affix(out, suffix)
    }
}

/// Writes `|v|` rounded to at most `fraction` digits (or `NaN`/`inf`) into
/// `buf` and returns whether the amount is negative. Amounts that round to
/// zero are never negative.
///
/// Like ICU, this rounds the shortest decimal representation of `v` half to
/// even, so `1.015` becomes `1.02` although the nearest `f64` is slightly
/// below 1.015.
fn round_abs<F: Float>(buf: &mut FloatStr, v: F, fraction: usize) -> Result<bool, fmt::Error> {
    if v.is_nan() {
        buf.write_str("NaN")?;
        return Ok(false);
    }
    if v.is_infinite() {
        buf.write_str("inf")?;
        return Ok(v.is_sign_negative());
    }
    write!(buf, "{}", v.abs())?;
    round_half_even(buf, fraction);
    let nonzero = buf.as_str().bytes().any(|b| (b'1'..=b'9').contains(&b));
    Ok(v.is_sign_negative() && nonzero)
}

/// Rounds the unsigned ASCII decimal in `buf` to at most `fraction` digits.
fn round_half_even(buf: &mut FloatStr, fraction: usize) {
    let Some(dot) = buf.as_str().find('.') else {
        return;
    };
    let bytes = buf.as_str().as_bytes();
    let Some(&first_dropped) = bytes.get(dot + 1 + fraction) else {
        return;
    };
    let last_kept = bytes[dot + fraction - usize::from(fraction == 0)];
    let rest_nonzero = bytes[dot + 2 + fraction..].iter().any(|&b| b != b'0');
    let up = first_dropped > b'5'
        || (first_dropped == b'5' && (rest_nonzero || (last_kept - b'0') % 2 == 1));

    buf.truncate(if fraction == 0 {
        dot
    } else {
        dot + 1 + fraction
    });
    if up && !buf.increment() {
        buf.prepend_one();
    }
}

fn split_decimal(s: &str) -> (&str, &str) {
    s.split_once('.').unwrap_or((s, ""))
}

/// An amount bound to a currency and locale. Formats itself via
/// [`Display`](fmt::Display).
#[derive(Debug, Clone, Copy)]
pub struct FormattedCurrency<N> {
    value: N,
    formatter: CurrencyFormatter,
}

impl<N: Number> WriteTo for FormattedCurrency<N> {
    fn write_to<W: Write + ?Sized>(&self, out: &mut W) -> fmt::Result {
        self.formatter.write(out, self.value.value())
    }
}

/// Honours width, fill and alignment (right-aligned by default).
impl<N: Number> fmt::Display for FormattedCurrency<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_util::display(self, f)
    }
}

/// Formats a number as a currency amount into a `String`.
pub trait ToCurrencyString {
    /// Formats `self` in the default currency of `locale`.
    fn to_currency(&self, locale: &Locale) -> String;

    /// Formats `self` in `currency`, localized for `locale`.
    fn to_currency_in(&self, locale: &Locale, currency: Currency) -> String;
}

impl<N: Number> ToCurrencyString for N {
    fn to_currency(&self, locale: &Locale) -> String {
        fmt_util::to_string(&CurrencyFormatter::new(*locale).format(*self))
    }

    fn to_currency_in(&self, locale: &Locale, currency: Currency) -> String {
        fmt_util::to_string(&CurrencyFormatter::with_currency(*locale, currency).format(*self))
    }
}
