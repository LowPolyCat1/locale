//! Locale-aware number formatting.
//!
//! [`NumberFormatter`] writes numbers with the separators, grouping and
//! native digits of a locale. The result is a [`FormattedNumber`], which
//! implements [`Display`](fmt::Display) (honouring width and alignment)
//! and writes into any formatter without heap allocations.
//!
//! ```
//! use locale_rs::Locale;
//! use locale_rs::nums::{NumberFormatter, ToFormattedString};
//!
//! assert_eq!(1234567.to_formatted_string(&Locale::de), "1.234.567");
//! assert_eq!(10000000.to_formatted_string(&Locale::hi), "1,00,00,000");
//!
//! let ar = NumberFormatter::new(Locale::ar_EG);
//! assert_eq!(format!("{}", ar.format(-1234.5)), "\u{61c}-١٬٢٣٤٫٥");
//! ```

use crate::Locale;
use crate::format::{
    self as fmt_util, FloatStr, WriteTo, u128_digits, write_digits, write_grouped,
};
use std::fmt::{self, Write};

pub use crate::data::{Grouping, NumberSymbols};

mod sealed {
    pub trait Sealed {}
}

/// The value of a [`Number`], split into sign and magnitude.
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub enum NumberValue {
    Int { negative: bool, abs: u128 },
    F32(f32),
    F64(f64),
}

/// A primitive number that can be formatted: every integer type, `f32`
/// and `f64`. This trait is sealed.
pub trait Number: Copy + sealed::Sealed {
    #[doc(hidden)]
    fn value(self) -> NumberValue;
}

macro_rules! impl_int {
    ($($t:ty),*) => {$(
        impl sealed::Sealed for $t {}
        impl Number for $t {
            #[inline]
            fn value(self) -> NumberValue {
                NumberValue::Int { negative: self < 0, abs: self.unsigned_abs() as u128 }
            }
        }
    )*};
}

macro_rules! impl_uint {
    ($($t:ty),*) => {$(
        impl sealed::Sealed for $t {}
        impl Number for $t {
            #[inline]
            fn value(self) -> NumberValue {
                NumberValue::Int { negative: false, abs: self as u128 }
            }
        }
    )*};
}

impl_int!(i8, i16, i32, i64, i128, isize);
impl_uint!(u8, u16, u32, u64, u128, usize);

impl sealed::Sealed for f32 {}
impl Number for f32 {
    #[inline]
    fn value(self) -> NumberValue {
        NumberValue::F32(self)
    }
}

impl sealed::Sealed for f64 {}
impl Number for f64 {
    #[inline]
    fn value(self) -> NumberValue {
        NumberValue::F64(self)
    }
}

/// Formats numbers for one locale.
#[derive(Debug, Clone, Copy)]
pub struct NumberFormatter {
    symbols: &'static NumberSymbols,
}

impl NumberFormatter {
    /// Creates a formatter for `locale`.
    pub fn new(locale: Locale) -> Self {
        Self {
            symbols: NumberSymbols::for_locale(locale),
        }
    }

    /// The symbols this formatter uses.
    pub fn symbols(&self) -> &'static NumberSymbols {
        self.symbols
    }

    /// Returns `value` wrapped for display.
    pub fn format<N: Number>(&self, value: N) -> FormattedNumber<N> {
        FormattedNumber {
            value,
            symbols: self.symbols,
        }
    }
}

/// A number bound to a locale. Formats itself via [`Display`](fmt::Display).
#[derive(Debug, Clone, Copy)]
pub struct FormattedNumber<N> {
    value: N,
    symbols: &'static NumberSymbols,
}

impl<N: Number> WriteTo for FormattedNumber<N> {
    fn write_to<W: Write + ?Sized>(&self, out: &mut W) -> fmt::Result {
        write_number(out, self.value.value(), self.symbols)
    }
}

/// Honours width, fill and alignment (right-aligned by default).
impl<N: Number> fmt::Display for FormattedNumber<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_util::display(self, f)
    }
}

/// Formats a number into a `String` for a locale.
pub trait ToFormattedString {
    /// Formats `self` with the number symbols of `locale`.
    fn to_formatted_string(&self, locale: &Locale) -> String;
}

impl<N: Number> ToFormattedString for N {
    fn to_formatted_string(&self, locale: &Locale) -> String {
        fmt_util::to_string(&NumberFormatter::new(*locale).format(*self))
    }
}

fn write_number<W: Write + ?Sized>(
    out: &mut W,
    value: NumberValue,
    symbols: &NumberSymbols,
) -> fmt::Result {
    match value {
        NumberValue::Int { negative, abs } => {
            if negative {
                out.write_str(symbols.minus_sign)?;
            }
            let mut buf = [0; 40];
            let int = u128_digits(&mut buf, abs);
            write_grouped(out, int, symbols.grouping, symbols.group, symbols.digits)
        }
        NumberValue::F32(v) => write_float(out, v, symbols),
        NumberValue::F64(v) => write_float(out, v, symbols),
    }
}

/// `f32` or `f64`.
pub(crate) trait Float: Copy + fmt::Display {
    fn is_nan(self) -> bool;
    fn is_infinite(self) -> bool;
    fn is_sign_negative(self) -> bool;
    fn abs(self) -> Self;
}

macro_rules! impl_float {
    ($($t:ty),*) => {$(
        impl Float for $t {
            fn is_nan(self) -> bool { <$t>::is_nan(self) }
            fn is_infinite(self) -> bool { <$t>::is_infinite(self) }
            fn is_sign_negative(self) -> bool { <$t>::is_sign_negative(self) }
            fn abs(self) -> Self { <$t>::abs(self) }
        }
    )*};
}

impl_float!(f32, f64);

/// Writes the shortest representation that round-trips, like `Display`.
fn write_float<W: Write + ?Sized, F: Float>(
    out: &mut W,
    v: F,
    symbols: &NumberSymbols,
) -> fmt::Result {
    if v.is_nan() {
        return out.write_str("NaN");
    }
    if v.is_sign_negative() {
        out.write_str(symbols.minus_sign)?;
    }
    if v.is_infinite() {
        return out.write_str("inf");
    }
    let mut abs = FloatStr::new();
    write!(abs, "{}", v.abs())?;
    write_decimal(out, abs.as_str(), symbols)
}

/// Writes an unsigned ASCII decimal such as `1234.5` with locale symbols.
pub(crate) fn write_decimal<W: Write + ?Sized>(
    out: &mut W,
    ascii: &str,
    symbols: &NumberSymbols,
) -> fmt::Result {
    let (int, frac) = match ascii.split_once('.') {
        Some((int, frac)) => (int, Some(frac)),
        None => (ascii, None),
    };
    write_grouped(out, int, symbols.grouping, symbols.group, symbols.digits)?;
    if let Some(frac) = frac {
        out.write_str(symbols.decimal)?;
        write_digits(out, frac, symbols.digits)?;
    }
    Ok(())
}
