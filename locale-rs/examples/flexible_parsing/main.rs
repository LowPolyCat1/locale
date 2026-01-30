use locale_rs::Locale;

fn main() {
    println!("=== Flexible Parsing ===\n");

    // Parse with different formats
    let formats = vec!["en-GB", "en_gb", "EN-gb", "en_GB", "en", "EN"];
    for format in formats {
        match Locale::from_flexible(format) {
            Ok(locale) => println!("✓ '{}' -> {}", format, locale),
            Err(e) => println!("✗ '{}' -> {}", format, e),
        }
    }

    println!("\n=== Locale Negotiation ===\n");

    // Negotiate best match from available locales
    let available = vec![Locale::en, Locale::de, Locale::fr];
    let user_preference = Locale::en_GB;

    match user_preference.negotiate(&available) {
        Some(best_match) => {
            println!(
                "User prefers: {}\nAvailable: {:?}\nBest match: {}",
                user_preference,
                available.iter().map(|l| l.as_str()).collect::<Vec<_>>(),
                best_match
            );
        }
        None => println!("No suitable locale found"),
    }

    println!("\n=== Locale Suggestions ===\n");

    // Get suggestions for typos
    let typo = "en-Gbb";
    let suggestions = Locale::suggest(typo);
    println!("Suggestions for '{}': ", typo);
    for (i, suggestion) in suggestions.iter().enumerate() {
        println!("  {}. {}", i + 1, suggestion);
    }

    println!("\n=== Language & Region Codes ===\n");

    // Extract language and region codes
    let locales = vec![Locale::en_GB, Locale::de_AT, Locale::zh_Hans, Locale::en];
    for locale in locales {
        let lang = locale.language_code();
        let region = locale.region_code();
        println!("{}: language='{}', region={:?}", locale, lang, region);
    }
}
