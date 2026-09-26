//! Locale-aware date and time formatting.
//!
//! The medium date and time patterns of every locale are parsed by
//! `locale-dev` at generation time, so formatting only walks a list of
//! [`DatePart`]s. Numeric fields use the locale's native digits.
//!
//! ```
//! use locale_rs::Locale;
//! use locale_rs::datetime::{DateTime, DateTimeFormatter};
//!
//! let dt = DateTime::new(2026, 1, 3, 14, 5, 9).unwrap();
//!
//! assert_eq!(dt.to_date_string(&Locale::en), "Jan 3, 2026");
//! assert_eq!(dt.to_date_string(&Locale::de), "03.01.2026");
//!
//! let ar = DateTimeFormatter::new(Locale::ar_EG);
//! assert_eq!(ar.format_time(&dt).to_string(), "٢:٠٥:٠٩ م");
//! ```

use crate::Locale;
use crate::data::NumberSymbols;
use crate::data::dates::DATE_SYMBOLS;
use crate::error::LocaleError;
use crate::format::{self as fmt_util, StackStr, WriteTo, write_digits, write_padded};
use std::fmt::{self, Write};

/// A calendar date and wall-clock time in the proleptic Gregorian calendar.
///
/// Fields are validated on construction, so formatting never fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DateTime {
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl DateTime {
    /// Creates a date and time, checking every field.
    ///
    /// ```
    /// use locale_rs::datetime::DateTime;
    ///
    /// assert!(DateTime::new(2024, 2, 29, 23, 59, 59).is_ok());
    /// assert!(DateTime::new(2023, 2, 29, 0, 0, 0).is_err());
    /// assert!(DateTime::new(2024, 13, 1, 0, 0, 0).is_err());
    /// ```
    pub fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, LocaleError> {
        fn check(
            field: &'static str,
            value: u8,
            range: std::ops::RangeInclusive<u8>,
        ) -> Result<(), LocaleError> {
            if range.contains(&value) {
                Ok(())
            } else {
                Err(LocaleError::InvalidDateTime {
                    field,
                    value: i64::from(value),
                })
            }
        }
        check("month", month, 1..=12)?;
        check("day", day, 1..=days_in_month(year, month))?;
        check("hour", hour, 0..=23)?;
        check("minute", minute, 0..=59)?;
        check("second", second, 0..=59)?;
        Ok(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        })
    }

    /// Creates a date at midnight.
    pub fn from_ymd(year: i32, month: u8, day: u8) -> Result<Self, LocaleError> {
        Self::new(year, month, day, 0, 0, 0)
    }

    /// The year; negative years count back from 1 BC = year 0.
    pub fn year(&self) -> i32 {
        self.year
    }

    /// The month, 1 to 12.
    pub fn month(&self) -> u8 {
        self.month
    }

    /// The day of the month, starting at 1.
    pub fn day(&self) -> u8 {
        self.day
    }

    /// The hour, 0 to 23.
    pub fn hour(&self) -> u8 {
        self.hour
    }

    /// The minute, 0 to 59.
    pub fn minute(&self) -> u8 {
        self.minute
    }

    /// The second, 0 to 59.
    pub fn second(&self) -> u8 {
        self.second
    }

    /// Day of the week, 0 = Sunday to 6 = Saturday.
    ///
    /// ```
    /// use locale_rs::datetime::DateTime;
    ///
    /// assert_eq!(DateTime::from_ymd(2026, 1, 3).unwrap().weekday(), 6);
    /// ```
    pub fn weekday(&self) -> u8 {
        // Sakamoto's algorithm, with Euclidean division for negative years.
        const T: [i64; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = i64::from(self.year);
        if self.month < 3 {
            y -= 1;
        }
        let days = y + y.div_euclid(4) - y.div_euclid(100)
            + y.div_euclid(400)
            + T[usize::from(self.month - 1)]
            + i64::from(self.day);
        days.rem_euclid(7) as u8
    }

    /// Formats the date with the medium date pattern of `locale`.
    pub fn to_date_string(&self, locale: &Locale) -> String {
        fmt_util::to_string(&DateTimeFormatter::new(*locale).format_date(self))
    }

    /// Formats the time with the medium time pattern of `locale`.
    pub fn to_time_string(&self, locale: &Locale) -> String {
        fmt_util::to_string(&DateTimeFormatter::new(*locale).format_time(self))
    }
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// One element of a parsed CLDR date pattern. Widths are the number of
/// repeated pattern letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DatePart {
    /// Literal text.
    Literal(&'static str),
    /// `y`: the year; `yy` is its last two digits.
    Year(u8),
    /// `M`: the month; numeric for widths 1-2, abbreviated name for 3,
    /// wide name for 4 and more.
    Month(u8),
    /// `d`: the day of the month.
    Day(u8),
    /// `H`: the hour, 0-23.
    Hour24(u8),
    /// `h`: the hour, 1-12.
    Hour12(u8),
    /// `m`: the minute.
    Minute(u8),
    /// `s`: the second.
    Second(u8),
    /// `a`: AM or PM.
    DayPeriod,
    /// `E`: the wide weekday name.
    Weekday,
}

/// A date or time pattern with its parsed form.
#[derive(Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct DatePattern {
    /// The CLDR pattern, e.g. `"MMM d, y"`.
    pub source: &'static str,
    /// The pattern split into fields and literals.
    pub parts: &'static [DatePart],
}

/// Gregorian calendar names and medium patterns of a locale.
#[derive(Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct DateSymbols {
    /// Month names, January first.
    pub months_wide: &'static [&'static str; 12],
    /// Abbreviated month names, January first.
    pub months_abbreviated: &'static [&'static str; 12],
    /// Weekday names, Sunday first.
    pub weekdays_wide: &'static [&'static str; 7],
    /// The AM marker.
    pub am: &'static str,
    /// The PM marker.
    pub pm: &'static str,
    /// The medium date pattern.
    pub date_pattern: &'static DatePattern,
    /// The medium time pattern.
    pub time_pattern: &'static DatePattern,
}

impl DateSymbols {
    /// Returns the date symbols of `locale`.
    pub fn for_locale(locale: Locale) -> &'static Self {
        DATE_SYMBOLS[locale.index()]
    }
}

/// Formats dates and times for one locale.
#[derive(Debug, Clone, Copy)]
pub struct DateTimeFormatter {
    symbols: &'static DateSymbols,
    digits: Option<&'static [char; 10]>,
}

impl DateTimeFormatter {
    /// Creates a formatter for `locale`.
    pub fn new(locale: Locale) -> Self {
        Self {
            symbols: DateSymbols::for_locale(locale),
            digits: NumberSymbols::for_locale(locale).digits,
        }
    }

    /// The symbols this formatter uses.
    pub fn symbols(&self) -> &'static DateSymbols {
        self.symbols
    }

    /// Returns the date of `dt` in the medium date pattern, wrapped for display.
    pub fn format_date(&self, dt: &DateTime) -> FormattedDateTime {
        self.format_pattern(dt, self.symbols.date_pattern)
    }

    /// Returns the time of `dt` in the medium time pattern, wrapped for display.
    pub fn format_time(&self, dt: &DateTime) -> FormattedDateTime {
        self.format_pattern(dt, self.symbols.time_pattern)
    }

    fn format_pattern(&self, dt: &DateTime, pattern: &'static DatePattern) -> FormattedDateTime {
        FormattedDateTime {
            formatter: *self,
            pattern,
            dt: *dt,
        }
    }

    fn write_number<W: Write + ?Sized>(&self, out: &mut W, n: i64, width: u8) -> fmt::Result {
        write_padded(out, n, width.into(), self.digits)
    }

    fn write<W: Write + ?Sized>(
        &self,
        out: &mut W,
        dt: &DateTime,
        pattern: &DatePattern,
    ) -> fmt::Result {
        let s = self.symbols;
        for part in pattern.parts {
            match *part {
                DatePart::Literal(text) => out.write_str(text)?,
                DatePart::Year(2) => {
                    let mut buf = StackStr::<16>::new();
                    write_padded(&mut buf, dt.year.into(), 0, None)?;
                    let year = buf.as_str();
                    if year.len() > 2 {
                        write_digits(out, &year[year.len() - 2..], self.digits)?;
                    } else {
                        self.write_number(out, dt.year.into(), 2)?;
                    }
                }
                DatePart::Year(w) => self.write_number(out, dt.year.into(), w)?,
                DatePart::Month(w @ (1 | 2)) => self.write_number(out, dt.month.into(), w)?,
                DatePart::Month(3) => {
                    out.write_str(s.months_abbreviated[usize::from(dt.month - 1)])?
                }
                DatePart::Month(_) => out.write_str(s.months_wide[usize::from(dt.month - 1)])?,
                DatePart::Day(w) => self.write_number(out, dt.day.into(), w)?,
                DatePart::Hour24(w) => self.write_number(out, dt.hour.into(), w)?,
                DatePart::Hour12(w) => {
                    let h = match dt.hour % 12 {
                        0 => 12,
                        h => h,
                    };
                    self.write_number(out, h.into(), w)?;
                }
                DatePart::Minute(w) => self.write_number(out, dt.minute.into(), w)?,
                DatePart::Second(w) => self.write_number(out, dt.second.into(), w)?,
                DatePart::DayPeriod => out.write_str(if dt.hour < 12 { s.am } else { s.pm })?,
                DatePart::Weekday => out.write_str(s.weekdays_wide[usize::from(dt.weekday())])?,
            }
        }
        Ok(())
    }
}

/// A date or time bound to a locale and pattern. Formats itself via
/// [`Display`](fmt::Display).
#[derive(Debug, Clone, Copy)]
pub struct FormattedDateTime {
    formatter: DateTimeFormatter,
    pattern: &'static DatePattern,
    dt: DateTime,
}

impl WriteTo for FormattedDateTime {
    fn write_to<W: Write + ?Sized>(&self, out: &mut W) -> fmt::Result {
        self.formatter.write(out, &self.dt, self.pattern)
    }
}

/// Honours width, fill and alignment (right-aligned by default).
impl fmt::Display for FormattedDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_util::display(self, f)
    }
}
