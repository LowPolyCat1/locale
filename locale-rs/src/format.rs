//! Allocation-free building blocks shared by the number, currency and date
//! formatters. Everything here writes straight into a [`fmt::Write`].

#[cfg(feature = "nums")]
use crate::data::Grouping;
use std::fmt::{self, Write};

/// Writes `ascii`, replacing the ASCII digits `0`-`9` with `digits`.
pub(crate) fn write_digits<W: Write + ?Sized>(
    out: &mut W,
    ascii: &str,
    digits: Option<&[char; 10]>,
) -> fmt::Result {
    let Some(digits) = digits else {
        return out.write_str(ascii);
    };
    for c in ascii.chars() {
        match c {
            '0'..='9' => out.write_char(digits[usize::from(c as u8 - b'0')])?,
            c => out.write_char(c)?,
        }
    }
    Ok(())
}

#[cfg(feature = "datetime")]
/// Writes `n` zero-padded to `width` characters including the sign, like
/// `{:0width$}`, with `digits` as native digits.
pub(crate) fn write_padded<W: Write + ?Sized>(
    out: &mut W,
    n: i64,
    width: usize,
    digits: Option<&[char; 10]>,
) -> fmt::Result {
    let mut buf = [0u8; 20];
    let mut pos = buf.len();
    let mut abs = n.unsigned_abs();
    loop {
        pos -= 1;
        buf[pos] = b'0' + (abs % 10) as u8;
        abs /= 10;
        if abs == 0 {
            break;
        }
    }
    let len = buf.len() - pos + usize::from(n < 0);
    if n < 0 {
        out.write_char('-')?;
    }
    for _ in len..width {
        write_digits(out, "0", digits)?;
    }
    // Only ASCII digits were written.
    write_digits(
        out,
        std::str::from_utf8(&buf[pos..]).unwrap_or_default(),
        digits,
    )
}

/// A formatted value that writes itself into any [`fmt::Write`]. Writing
/// is generic, so a `String` target compiles to plain pushes.
pub(crate) trait WriteTo {
    fn write_to<W: Write + ?Sized>(&self, out: &mut W) -> fmt::Result;
}

/// Implements `Display` for a [`WriteTo`]: renders into a stack buffer and
/// pads it to the formatter's width. Too long output is written unpadded.
pub(crate) fn display<T: WriteTo>(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut buf = StackStr::<256>::new();
    if value.write_to(&mut buf).is_err() {
        return value.write_to(f);
    }
    let s = buf.as_str();
    let Some(width) = f.width() else {
        return f.write_str(s);
    };
    let len = s.chars().count();
    let pad = width.saturating_sub(len);
    // Numbers are right-aligned by default.
    let (before, after) = match f.align() {
        Some(fmt::Alignment::Left) => (0, pad),
        Some(fmt::Alignment::Center) => (pad / 2, pad - pad / 2),
        _ => (pad, 0),
    };
    let fill = f.fill();
    for _ in 0..before {
        f.write_char(fill)?;
    }
    f.write_str(s)?;
    for _ in 0..after {
        f.write_char(fill)?;
    }
    Ok(())
}

/// Renders a [`WriteTo`] into a new `String`.
pub(crate) fn to_string<T: WriteTo>(value: &T) -> String {
    let mut s = String::with_capacity(32);
    // Writing into a `String` cannot fail.
    let _ = value.write_to(&mut s);
    s
}

#[cfg(feature = "nums")]
/// Writes the ASCII integer digits `int`, inserting `separator` according to
/// `grouping` and mapping digits to `digits`.
pub(crate) fn write_grouped<W: Write + ?Sized>(
    out: &mut W,
    int: &str,
    grouping: Grouping,
    separator: &str,
    digits: Option<&[char; 10]>,
) -> fmt::Result {
    let primary = usize::from(grouping.primary);
    let secondary = usize::from(grouping.secondary).max(1);
    if primary == 0 || int.len() <= primary {
        return write_digits(out, int, digits);
    }

    // Length of the leftmost group; every later group but the last has
    // `secondary` digits and the last one has `primary`.
    let head = int.len() - primary;
    let first = match head % secondary {
        0 => secondary,
        n => n,
    };
    write_digits(out, &int[..first], digits)?;
    let mut pos = first;
    while pos < head {
        out.write_str(separator)?;
        write_digits(out, &int[pos..pos + secondary], digits)?;
        pos += secondary;
    }
    out.write_str(separator)?;
    write_digits(out, &int[head..], digits)
}

#[cfg(feature = "nums")]
/// Writes `n` in ASCII decimal into `buf` and returns the digits.
pub(crate) fn u128_digits(buf: &mut [u8; 40], mut n: u128) -> &str {
    let mut pos = buf.len();
    loop {
        pos -= 1;
        buf[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    // Only ASCII digits were written.
    std::str::from_utf8(&buf[pos..]).unwrap_or_default()
}

/// A fixed-capacity string on the stack. Writing past the capacity fails
/// with [`fmt::Error`] instead of allocating.
pub(crate) struct StackStr<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> StackStr<N> {
    pub(crate) const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        // Only whole `&str`s and ASCII digits are ever written.
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or_default()
    }

    /// Shortens the string to `len` bytes, which must be a char boundary.
    #[cfg(feature = "currency")]
    pub(crate) fn truncate(&mut self, len: usize) {
        self.len = self.len.min(len);
    }

    /// Adds one to the ASCII decimal in the buffer, skipping a `.`. Returns
    /// `false` if the carry ran past the first digit (`99.9` became `00.0`).
    #[cfg(feature = "currency")]
    pub(crate) fn increment(&mut self) -> bool {
        for b in self.buf[..self.len].iter_mut().rev() {
            match *b {
                b'.' => {}
                b'9' => *b = b'0',
                _ => {
                    *b += 1;
                    return true;
                }
            }
        }
        false
    }

    /// Inserts a `1` in front, dropping the last byte if the buffer is full.
    #[cfg(feature = "currency")]
    pub(crate) fn prepend_one(&mut self) {
        let len = (self.len + 1).min(N);
        self.buf.copy_within(0..len - 1, 1);
        self.buf[0] = b'1';
        self.len = len;
    }
}

impl<const N: usize> Write for StackStr<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        if end > N {
            return Err(fmt::Error);
        }
        self.buf[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// Large enough for the `Display` output of any `f64`, including
/// `f64::MIN_POSITIVE / 2.0` written out in full.
#[cfg(feature = "nums")]
pub(crate) type FloatStr = StackStr<512>;

#[cfg(all(test, feature = "nums"))]
mod tests {
    use super::*;

    fn grouped(int: &str, primary: u8, secondary: u8) -> String {
        let mut s = String::new();
        write_grouped(&mut s, int, Grouping { primary, secondary }, ",", None).unwrap();
        s
    }

    #[test]
    fn grouping_western_and_indian() {
        assert_eq!(grouped("1", 3, 3), "1");
        assert_eq!(grouped("123", 3, 3), "123");
        assert_eq!(grouped("1234", 3, 3), "1,234");
        assert_eq!(grouped("123456789", 3, 3), "123,456,789");
        assert_eq!(grouped("10000000", 3, 2), "1,00,00,000");
        assert_eq!(grouped("123456", 3, 2), "1,23,456");
        assert_eq!(grouped("12345", 0, 0), "12345");
    }

    #[test]
    fn digits_are_mapped() {
        let arab = ['٠', '١', '٢', '٣', '٤', '٥', '٦', '٧', '٨', '٩'];
        let mut s = String::new();
        write_digits(&mut s, "a1-20", Some(&arab)).unwrap();
        assert_eq!(s, "a١-٢٠");
    }

    #[test]
    #[cfg(feature = "datetime")]
    fn padded_matches_std() {
        for (n, width) in [
            (5, 2),
            (-5, 3),
            (2026, 1),
            (0, 4),
            (-2026, 2),
            (i64::MIN, 25),
        ] {
            let mut s = String::new();
            write_padded(&mut s, n, width, None).unwrap();
            assert_eq!(s, format!("{n:0width$}"));
        }
    }

    #[test]
    fn u128_extremes() {
        let mut buf = [0; 40];
        assert_eq!(u128_digits(&mut buf, 0), "0");
        assert_eq!(u128_digits(&mut buf, u128::MAX), u128::MAX.to_string());
    }

    #[test]
    fn stack_str_rejects_overflow() {
        let mut s = StackStr::<4>::new();
        assert!(s.write_str("abcd").is_ok());
        assert!(s.write_str("e").is_err());
        assert_eq!(s.as_str(), "abcd");
    }

    #[test]
    fn float_str_fits_every_f64() {
        let mut s = FloatStr::new();
        write!(s, "{}", f64::MIN_POSITIVE / 2.0).unwrap();
        let mut s = FloatStr::new();
        write!(s, "{}", f64::MAX).unwrap();
    }
}
