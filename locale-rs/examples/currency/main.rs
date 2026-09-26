use locale_rs::{Currency, CurrencyFormatter, Locale, ToCurrencyString};

fn main() {
    // Each locale formats its own default currency.
    for locale in [
        Locale::en,
        Locale::de,
        Locale::de_CH,
        Locale::en_IN,
        Locale::ja,
    ] {
        println!("{locale:>6}: {}", 1234567.891.to_currency(&locale));
    }

    // Any currency can be formatted for any locale.
    let eur: Currency = "EUR".parse().unwrap();
    for locale in [Locale::en, Locale::fr, Locale::nl] {
        let formatter = CurrencyFormatter::with_currency(locale, eur);
        println!(
            "{locale:>6}: {} / {}",
            formatter.format(-42),
            formatter.format(0.5)
        );
    }
}
