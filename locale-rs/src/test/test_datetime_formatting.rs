use crate::AVAILABLE_LOCALES;
use crate::datetime::{DatePart, DateSymbols, DateTime, DateTimeFormatter};
use crate::error::LocaleError;
use crate::locale::Locale;
use std::str::FromStr;

fn base_dt() -> DateTime {
    DateTime::new(2026, 1, 3, 14, 5, 9).unwrap()
}

#[test]
fn test_english() {
    assert_eq!(base_dt().to_date_string(&Locale::en), "Jan 3, 2026");
    assert_eq!(base_dt().to_time_string(&Locale::en), "2:05:09\u{202f}PM");
}

#[test]
fn test_literals_via_chinese_locale() {
    assert_eq!(base_dt().to_date_string(&Locale::zh_Hans), "2026年1月3日");
}

#[test]
fn test_german() {
    assert_eq!(base_dt().to_date_string(&Locale::de), "03.01.2026");
    assert_eq!(base_dt().to_time_string(&Locale::de), "14:05:09");
}

#[test]
fn test_native_digits() {
    let ar = DateTimeFormatter::new(Locale::ar_EG);
    assert_eq!(
        ar.format_date(&base_dt()).to_string(),
        "٠٣\u{200f}/٠١\u{200f}/٢٠٢٦"
    );
    assert_eq!(ar.format_time(&base_dt()).to_string(), "٢:٠٥:٠٩ م");
}

#[test]
fn test_parsed_patterns() {
    let en = DateSymbols::for_locale(Locale::en);
    assert_eq!(en.date_pattern.source, "MMM d, y");
    assert_eq!(
        en.date_pattern.parts,
        [
            DatePart::Month(3),
            DatePart::Literal(" "),
            DatePart::Day(1),
            DatePart::Literal(", "),
            DatePart::Year(1)
        ]
    );
    assert_eq!(en.months_wide[0], "January");
    assert_eq!(en.weekdays_wide[0], "Sunday");
}

#[test]
fn test_every_locale_formats() {
    let dt = base_dt();
    for locale in AVAILABLE_LOCALES {
        let loc = Locale::from_str(locale).unwrap();
        assert!(!dt.to_date_string(&loc).is_empty());
        assert!(!dt.to_time_string(&loc).is_empty());
    }
}

#[test]
fn test_validation() {
    assert!(DateTime::new(2024, 2, 29, 0, 0, 0).is_ok());
    assert!(DateTime::new(2000, 2, 29, 0, 0, 0).is_ok());
    assert_eq!(
        DateTime::new(1900, 2, 29, 0, 0, 0),
        Err(LocaleError::InvalidDateTime {
            field: "day",
            value: 29
        })
    );
    assert_eq!(
        DateTime::new(2026, 0, 1, 0, 0, 0),
        Err(LocaleError::InvalidDateTime {
            field: "month",
            value: 0
        })
    );
    assert!(DateTime::new(2026, 4, 31, 0, 0, 0).is_err());
    assert!(DateTime::new(2026, 1, 1, 24, 0, 0).is_err());
    assert!(DateTime::new(2026, 1, 1, 0, 60, 0).is_err());
    assert!(DateTime::new(2026, 1, 1, 0, 0, 60).is_err());
}

#[test]
fn test_weekday() {
    assert_eq!(DateTime::from_ymd(2026, 1, 3).unwrap().weekday(), 6);
    assert_eq!(DateTime::from_ymd(2000, 1, 1).unwrap().weekday(), 6);
    assert_eq!(DateTime::from_ymd(1970, 1, 1).unwrap().weekday(), 4);
    // Proleptic Gregorian: 1 BC (year 0) ended on a Sunday.
    assert_eq!(DateTime::from_ymd(0, 12, 31).unwrap().weekday(), 0);
    assert!(DateTime::from_ymd(-4000, 3, 1).unwrap().weekday() < 7);
}

#[test]
fn test_accessors() {
    let dt = base_dt();
    assert_eq!(
        (
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second()
        ),
        (2026, 1, 3, 14, 5, 9)
    );
}
