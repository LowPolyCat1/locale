use locale_rs::Locale;
use locale_rs::datetime::{DateSymbols, DateTime, DateTimeFormatter};

fn main() {
    let dt = DateTime::new(2026, 1, 3, 14, 5, 9).expect("valid date");

    for locale in [Locale::en, Locale::de, Locale::zh_Hans, Locale::ar_EG] {
        let formatter = DateTimeFormatter::new(locale);
        println!(
            "{locale:>7}: {}  {}",
            formatter.format_date(&dt),
            formatter.format_time(&dt)
        );
    }

    // The underlying CLDR data is available too.
    let fr = DateSymbols::for_locale(Locale::fr);
    println!("fr months: {}", fr.months_wide.join(", "));
    println!("fr date pattern: {}", fr.date_pattern.source);

    // Invalid dates are rejected up front.
    println!("{:?}", DateTime::new(2026, 2, 30, 0, 0, 0));
}
