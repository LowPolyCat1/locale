# locale-rs

A strongly-typed Rust library for Unicode locales, built directly on the **CLDR (Common Locale Data Repository)** dataset.

`Locale` is a plain enum with one variant per CLDR locale, so an invalid locale identifier is a compile error rather than a runtime surprise. Optional features add locale-aware number, currency and date formatting on top.

## Features

- **<!-- gen:{{locale_count}} -->766<!-- /gen --> Unicode Locales**: Complete coverage of CLDR <!-- gen:{{cldr_version}} -->48.2.2<!-- /gen -->
- **Type-Safe Locales**: Locale identifiers are enum variants, checked at compile time
- **CLDR Inheritance**: Fallback chains follow CLDR parent locales (`en-IN` inherits from `en-001`)
- **Number Formatting**: Separators, grouping (including Indian `12,34,567`) and native digits
- **Currency Formatting**: CLDR currency patterns, symbols and fraction digits for any ISO 4217 currency
- **DateTime Formatting**: Localized month and weekday names with the medium date and time patterns
- **Allocation-Free Display**: Formatters return values that implement `Display` and write straight into any buffer
- **Flexible Parsing, Negotiation and Suggestions**: Case-insensitive parsing, best-match negotiation and typo suggestions

## Quick Start

### Installation

Add to your `Cargo.toml`:

<!-- gen:
```toml
[dependencies]
locale-rs = "{{crate_version_req}}"

# With number formatting support
locale-rs = { version = "{{crate_version_req}}", features = ["nums"] }

# With all features
locale-rs = { version = "{{crate_version_req}}", features = ["all"] }
```
-->
```toml
[dependencies]
locale-rs = "0.4.0-rc.1"

# With number formatting support
locale-rs = { version = "0.4.0-rc.1", features = ["nums"] }

# With all features
locale-rs = { version = "0.4.0-rc.1", features = ["all"] }
```
<!-- /gen -->

### Locales

```rust
use locale_rs::Locale;

// Direct enum access
let locale = Locale::en_GB;
assert_eq!(locale.to_string(), "en-GB");

// Parsing is case-insensitive and accepts `_` or `-`
let parsed: Locale = "en_gb".parse().unwrap();
assert_eq!(parsed, Locale::en_GB);

// Subtags
assert_eq!(locale.language_code(), "en");
assert_eq!(locale.region_code(), Some("GB"));

// CLDR fallback chain
let chain: Vec<Locale> = Locale::en_IN.fallback_chain().collect();
assert_eq!(chain, [Locale::en_IN, Locale::en_001, Locale::en]);

// Negotiation walks the chain
let available = [Locale::en, Locale::de, Locale::fr];
assert_eq!(Locale::en_GB.negotiate(&available), Some(Locale::en));

// Suggestions for typos
assert!(Locale::suggest("en-gbb").contains(&Locale::en_GB));
```

### Numbers (`nums`)

```rust
use locale_rs::Locale;
use locale_rs::nums::{NumberFormatter, NumberSymbols, ToFormattedString};

assert_eq!(1234567.to_formatted_string(&Locale::en), "1,234,567");
assert_eq!(1234567.to_formatted_string(&Locale::de), "1.234.567");
assert_eq!(1234567.to_formatted_string(&Locale::hi), "12,34,567");
assert_eq!(1234567.to_formatted_string(&Locale::ar_EG), "١٬٢٣٤٬٥٦٧");
assert_eq!(42.5.to_formatted_string(&Locale::de), "42,5");

// A formatter is reusable and its output implements `Display`,
// including width and alignment.
let de = NumberFormatter::new(Locale::de);
assert_eq!(format!("[{:>10}]", de.format(-1234.5)), "[  -1.234,5]");

// The underlying CLDR data
let symbols = NumberSymbols::for_locale(Locale::de_CH);
assert_eq!((symbols.decimal, symbols.group), (".", "'"));
```

### Currency (`currency`)

```rust
use locale_rs::Locale;
use locale_rs::currency::{Currency, CurrencyFormatter, ToCurrencyString};

// Each locale defaults to the currency of its country
assert_eq!(1234.5.to_currency(&Locale::en), "$1,234.50");
assert_eq!(1234.5.to_currency(&Locale::de), "1.234,50\u{a0}€");
assert_eq!(1234.5.to_currency(&Locale::en_IN), "₹1,234.50");
assert_eq!(Currency::default_for(Locale::de_CH).as_str(), "CHF");

// Any currency in any locale, with its own fraction digits
let yen: Currency = "JPY".parse().unwrap();
assert_eq!(1234.5.to_currency_in(&Locale::en, yen), "¥1,234");

// Negative patterns come from CLDR
let chf = CurrencyFormatter::new(Locale::de_CH);
assert_eq!(chf.format(-5).to_string(), "CHF-5.00");
```

Floating-point amounts are rounded half to even on their shortest decimal representation, as ICU does. Integer amounts are formatted exactly, however large.

### Dates and Times (`datetime`)

```rust
use locale_rs::Locale;
use locale_rs::datetime::{DateSymbols, DateTime, DateTimeFormatter};

// Fields are validated up front
let dt = DateTime::new(2026, 1, 3, 14, 5, 9).unwrap();
assert!(DateTime::new(2026, 2, 30, 0, 0, 0).is_err());

assert_eq!(dt.to_date_string(&Locale::en), "Jan 3, 2026");
assert_eq!(dt.to_date_string(&Locale::de), "03.01.2026");
assert_eq!(dt.to_date_string(&Locale::zh_Hans), "2026年1月3日");

// Numeric fields use native digits
let ar = DateTimeFormatter::new(Locale::ar_EG);
assert_eq!(ar.format_time(&dt).to_string(), "٢:٠٥:٠٩ م");

// The underlying CLDR data, with pre-parsed patterns
let de = DateSymbols::for_locale(Locale::de);
assert_eq!(de.months_wide[0], "Januar");
assert_eq!(de.date_pattern.source, "dd.MM.y");
```

### Iterating Locales (`strum`)

```rust
use locale_rs::Locale;
use strum::IntoEnumIterator;

assert_eq!(Locale::iter().count(), locale_rs::AVAILABLE_LOCALES.len());
```

## Features

None of these are enabled by default.

<!-- gen:
{{feature_table}}
-->
| Feature | Enables | Description |
| --- | --- | --- |
| `strum` | `strum` crate, `strum_macros` crate | Derives `strum` traits on `Locale`, e.g. iterating over all locales. |
| `datetime` | - | Localized date and time formatting (`datetime` module). |
| `nums` | - | Locale-aware number formatting with native digits (`nums` module). |
| `currency` | `nums` | Currency formatting from CLDR currency patterns (`currency` module). |
| `all` | `datetime`, `nums`, `strum`, `currency` | Every feature above. |
<!-- /gen -->

`Locale` itself has the same API whatever features are enabled; the locale data is reached through the types of each feature module.

## API Overview

| Module | Types | Purpose |
| --- | --- | --- |
| crate root | `Locale`, `LocaleError`, `AVAILABLE_LOCALES`, `CLDR_VERSION` | Identifiers, parsing, fallback, negotiation |
| `nums` | `NumberFormatter`, `FormattedNumber`, `NumberSymbols`, `Grouping`, `ToFormattedString` | Number formatting |
| `currency` | `CurrencyFormatter`, `FormattedCurrency`, `Currency`, `ToCurrencyString` | Currency formatting |
| `datetime` | `DateTimeFormatter`, `FormattedDateTime`, `DateTime`, `DateSymbols`, `DatePattern`, `DatePart` | Date and time formatting |

## Error Handling

```rust
use locale_rs::{Locale, LocaleError};

match "invalid-locale".parse::<Locale>() {
    Ok(locale) => println!("Valid: {locale}"),
    Err(LocaleError::UnknownLocale(s)) => println!("Unknown locale: {s}"),
    Err(e) => println!("Error: {e}"),
}
```

`LocaleError` has three variants: `UnknownLocale`, `InvalidCurrency` and `InvalidDateTime`.

## Stability

All public enums, `Locale` included, are exhaustive on purpose: when a release adds or removes a variant, every `match` that needs attention becomes a compile error. In exchange, every such change, and every CLDR data update that changes output, is released as a breaking version, so it never reaches you through a plain `cargo update`.

## Architecture

All CLDR data lives in `src/data/`, which is generated by [`locale-dev`](../locale-dev/README.md) and contains nothing but static tables indexed by `Locale`. Identical values are stored once. Date and currency patterns are parsed at generation time, so formatting only walks pre-parsed structures. Everything outside `src/data/` is handwritten Rust.

To regenerate the data:

```bash
# In the workspace root: fetch the latest CLDR release if it is newer
cargo run -p locale-dev

# Or regenerate from a local cldr-json archive, e.g. after changing the generator
cargo run -p locale-dev -- --archive path/to/cldr-48.2.2-json-full.zip
```

## Licensing

This project respects and adheres to the licensing requirements of its source data:

- **Data Source**: All locale data is derived from the Unicode CLDR project and is subject to the **[Unicode License V3](https://www.unicode.org/license.txt)**.
- **Code Inspiration**: Architectural patterns inspired by [`num-format`](https://github.com/bcmyers/num-format), dual-licensed under **Apache-2.0** or **MIT**.
- **This Project**: Licensed under **[MIT License](../LICENSE-MIT)** or **[Apache-2.0 License](../LICENSE-APACHE)**.

## Contributing

Contributions are welcome! Formatting logic lives in handwritten modules (`src/nums.rs`, `src/currency.rs`, `src/datetime.rs`); data changes go through `locale-dev`, never by editing `src/data/` by hand.

If you find a missing locale or discrepancy with CLDR standards, please open an issue.

## See Also

- [locale-dev](../locale-dev/README.md) - Code generation tool
- [CLDR Project](https://cldr.unicode.org/) - Unicode locale data source
- [CLDR-JSON Repository](https://github.com/unicode-org/cldr-json) - GitHub source
- [num-format](https://github.com/bcmyers/num-format) - Inspiration for this library
