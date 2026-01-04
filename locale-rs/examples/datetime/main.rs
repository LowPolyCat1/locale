#[cfg(feature = "datetime")]
fn main() {
    use locale_rs::{datetime_formats::DateTime, locale::Locale};
    // Create a base DateTime object
    let dt = DateTime {
        year: 2026,
        month: 1,
        day: 3,
        hour: 14,
        minute: 5,
        second: 9,
    };

    // 1. Standard English (US) formatting
    let locale = Locale::en;
    println!("English (US):");
    println!("  Date: {}", locale.format_date(&dt)); // e.g., Jan 3, 2026
    println!("  Time: {}\n", locale.format_time(&dt)); // e.g., 14:05:09

    // 2. German (DE) - Uses '.' for dates and 'yy' truncation
    let locale = Locale::de;
    println!("German (DE):");
    println!("  Date: {}\n", locale.format_date(&dt)); // e.g., 03.01.26

    // 3. Chinese (Simplified) - Handles literal characters (年, 月, 日)
    let locale = Locale::zh_Hans;
    println!("Chinese (Simplified):");
    println!("  Date: {}\n", locale.format_date(&dt)); // e.g., 2026年1月3日

    // 4. Arabic (EG) - Demonstrates native digit translation
    let locale = Locale::ar_EG;
    println!("Arabic (Egypt):");
    println!("  Date: {}", locale.format_date(&dt));
    println!("  Time: {}", locale.format_time(&dt));
}

#[cfg(not(feature = "datetime"))]
fn main() {
    panic!("Feature datetime is not enabled")
}
