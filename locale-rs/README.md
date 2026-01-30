# locale-rs

A comprehensive, strongly-typed Rust library for managing Unicode locales, built directly on the **CLDR (Common Locale Data Repository)** dataset.

This crate provides a type-safe interface for locale identifiers, ensuring that your application remains compliant with international standards while benefiting from Rust's performance and safety guarantees.

## Features

- **766 Unicode Locales**: Complete coverage of CLDR 48.1.0
- **Type-Safe Locales**: Compile-time validated locale identifiers as Rust enums
- **Zero-Cost Abstractions**: No runtime overhead for locale operations
- **Number Formatting**: Locale-aware formatting with native digit support
- **Currency Formatting**: ICU-compatible currency patterns
- **DateTime Formatting**: Localized month/weekday names and patterns
- **Native Numbering Systems**: Automatic support for Arabic-Indic, Devanagari, Bengali, and more
- **Flexible Parsing**: Parse locales with hyphens, underscores, or mixed case
- **Locale Negotiation**: Find the best matching locale from available options
- **Fuzzy Suggestions**: Get locale suggestions for typos or unknown identifiers

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
locale-rs = "0.1"

# With number formatting support
locale-rs = { version = "0.1", features = ["nums"] }

# With all features
locale-rs = { version = "0.1", features = ["all"] }
```

### Basic Usage

```rust
use locale_rs::Locale;

// Direct enum access
let locale = Locale::en_GB;
println!("{}", locale);  // "en-GB"

// Parse from string
let locale = Locale::from_str("en-GB")?;

// Flexible parsing (case-insensitive, accepts hyphens or underscores)
let locale = Locale::from_flexible("en_gb")?;

// Extract subtags
assert_eq!(locale.language_code(), "en");
assert_eq!(locale.region_code(), Some("GB"));
```

### Number Formatting

```rust
use locale_rs::Locale;
use locale_rs::num_formats::ToFormattedString;

let num = 1234567;

// English: 1,234,567
println!("{}", num.to_formatted_string(&Locale::en));

// German: 1.234.567
println!("{}", num.to_formatted_string(&Locale::de));

// French: 1 234 567
println!("{}", num.to_formatted_string(&Locale::fr));

// Arabic with native digits: ١٬٢٣٤٬٥٦٧
println!("{}", num.to_formatted_string(&Locale::ar));
```

### Locale Negotiation

```rust
use locale_rs::Locale;

let user_preference = Locale::en_GB;
let available = vec![Locale::en, Locale::de, Locale::fr];

// Find best match (falls back through parent chain)
if let Some(best) = user_preference.negotiate(&available) {
    println!("Using: {}", best);  // "en"
}
```

### Locale Suggestions

```rust
use locale_rs::Locale;

// Get suggestions for typos or unknown locales
let suggestions = Locale::suggest("en-gbb");
for locale in suggestions {
    println!("{}", locale);  // Suggests: en-GB, en, etc.
}
```

## Features

### `nums` - Number Formatting

Enables number formatting with locale-specific separators and native digit systems.

```rust
use locale_rs::Locale;
use locale_rs::num_formats::ToFormattedString;

let value = 42.5;
println!("{}", value.to_formatted_string(&Locale::de));  // 42,5
```

### `currency` - Currency Formatting

Enables currency formatting patterns (requires `nums`).

```rust
use locale_rs;
use locale_rs::currency_formats::ToCurrencyString;
let locale = locale_rs::Locale::en;
for i in 0u32..10 {
    println!("{}", i.to_currency(&locale))
// $0,-
// $1,-
// $2,-
// $3,-
// $4,-
// $5,-
// $6,-
// $7,-
// $8,-
// $9,-
}
let locale = locale_rs::Locale::de;
for i in 0u32..10 {
    println!("{}", i.to_currency(&locale))
// 0,- €
// 1,- €
// 2,- €
// 3,- €
// 4,- €
// 5,- €
// 6,- €
// 7,- €
// 8,- €
// 9,- €
}
```

### `datetime` - DateTime Formatting

Enables datetime formatting data.

```rust
use locale_rs::Locale;

let locale = Locale::de;
let months = locale.months_wide();
println!("{}", months[0]);  // "Januar"
```

### `strum` - Enum Iteration

Enables iteration over all locales using the `strum` crate.

```rust
use locale_rs::Locale;
use strum::IntoEnumIter;

for locale in Locale::iter() {
    println!("{}", locale);
}
```

### `all` - All Features

Enables all optional features.

```toml
locale-rs = { version = "0.1", features = ["all"] }
```

## API Overview

### Core Methods

| Method | Returns | Purpose |
|--------|---------|---------|
| `as_str()` | `&'static str` | Get string representation |
| `fallback()` | `Option<Locale>` | Get parent locale |
| `language_code()` | `&'static str` | Extract language subtag |
| `region_code()` | `Option<&'static str>` | Extract region subtag |
| `from_flexible(s)` | `Result<Locale, LocaleError>` | Parse with flexible formatting |
| `negotiate(available)` | `Option<Locale>` | Find best match from list |
| `suggest(input)` | `Vec<Locale>` | Get fuzzy suggestions |

### Number Formatting (with `nums` feature)

| Method | Returns | Purpose |
|--------|---------|---------|
| `decimal_separator()` | `&'static str` | Decimal point character |
| `grouping_separator()` | `&'static str` | Thousands separator |
| `grouping_sizes()` | `&'static [usize]` | Grouping size array |
| `minus_sign()` | `&'static str` | Negative sign character |
| `digits()` | `Option<[char; 10]>` | Native digit characters |

### Currency Formatting (with `currency` feature)

| Method | Returns | Purpose |
|--------|---------|---------|
| `currency_standard_pattern()` | `&'static str` | Standard currency pattern |
| `currency_accounting_pattern()` | `&'static str` | Accounting format pattern |

### DateTime Formatting (with `datetime` feature)

| Method | Returns | Purpose |
|--------|---------|---------|
| `months_wide()` | `&'static [&'static str]` | Full month names |
| `months_abbreviated()` | `&'static [&'static str]` | Short month names |
| `weekdays_wide()` | `&'static [&'static str]` | Full weekday names |
| `weekdays_abbreviated()` | `&'static [&'static str]` | Short weekday names |

## Examples

### Parsing Locales

```rust
use locale_rs::Locale;
use std::str::FromStr;

// Standard parsing
let locale = Locale::from_str("en-GB")?;

// Flexible parsing (case-insensitive, accepts underscores)
let locale = Locale::from_flexible("en_gb")?;
let locale = Locale::from_flexible("EN-GB")?;

// TryFrom conversion
let locale = Locale::try_from("en-GB")?;
```

### Fallback Chain

```rust
use locale_rs::Locale;

let mut current = Locale::en_GB;
while let Some(parent) = current.fallback() {
    println!("Fallback: {}", parent);
    current = parent;
}
// Output:
// Fallback: en
```

### Locale Matching

```rust
use locale_rs::Locale;

let user_locales = vec![Locale::en_GB, Locale::en];
let available = vec![Locale::en, Locale::de, Locale::fr];

// Find best match for each user locale
for user_locale in user_locales {
    if let Some(best) = user_locale.negotiate(&available) {
        println!("{} -> {}", user_locale, best);
    }
}
// Output:
// en-GB -> en
// en -> en
```

### Formatting Numbers

```rust
use locale_rs::Locale;
use locale_rs::num_formats::ToFormattedString;

let numbers = vec![1000, 1000000, 1234567];

for num in numbers {
    println!("en: {}", num.to_formatted_string(&Locale::en));
    println!("de-DE: {}", num.to_formatted_string(&Locale::de));
    println!("fr-FR: {}", num.to_formatted_string(&Locale::fr));
    println!("ar-SA: {}", num.to_formatted_string(&Locale::ar_SA));
    println!();
}
```

### Formatting Floats

```rust
use locale_rs::Locale;
use locale_rs::num_formats::ToFormattedString;

let value = 3.14159;

println!("en: {}", value.to_formatted_string(&Locale::en));      // 3.14159
println!("de-DE: {}", value.to_formatted_string(&Locale::de));     // 3,14159
println!("fr-FR: {}", value.to_formatted_string(&Locale::fr));     // 3,14159
```

### Currency Patterns

```rust
use locale_rs::Locale;

let locales = vec![
    Locale::en,
    Locale::de,
    Locale::fr,
    Locale::ja,
];

for locale in locales {
    let pattern = locale.currency_standard_pattern();
    println!("{}: {}", locale, pattern);
}
// Output:
// en: ¤#,##0.00
// de: #,##0.00 ¤
// fr: #,##0.00 ¤
// ja: ¤#,##0.00
```

### DateTime Data

```rust
use locale_rs::Locale;

let locale = Locale::de;

println!("Months:");
for (i, month) in locale.months_wide().iter().enumerate() {
    println!("  {}: {}", i + 1, month);
}

println!("\nWeekdays:");
for (i, day) in locale.weekdays_abbreviated().iter().enumerate() {
    println!("  {}: {}", i, day);
}
```

## Supported Locales

The library supports **766 locales** from CLDR 48.1.0, including:

- **Languages**: 200+ languages
- **Regions**: 150+ territories
- **Scripts**: Multiple script variants (e.g., `zh-Hans`, `zh-Hant`)
- **Variants**: Special variants (e.g., `ca-ES-valencia`, `be-tarask`)

View all available locales:

```rust
use locale_rs::AVAILABLE_LOCALES;

for locale_str in AVAILABLE_LOCALES.iter() {
    println!("{}", locale_str);
}
```

## Performance

- **Runtime**: Zero-cost abstractions; all operations are compile-time validated
- **Memory**: Locale enum variants are zero-sized types

## Error Handling

```rust
use locale_rs::{Locale, LocaleError};
use std::str::FromStr;

match Locale::from_str("invalid-locale") {
    Ok(locale) => println!("Valid: {}", locale),
    Err(LocaleError::UnknownLocale(s)) => println!("Unknown locale: {}", s),
    Err(LocaleError::Unknown(msg)) => println!("Error: {}", msg),
}
```

## Architecture

This library is auto-generated from Unicode CLDR data by the `locale-dev` tool. The generation process:

1. Fetches the latest CLDR-JSON release from GitHub
2. Parses locale definitions and formatting rules
3. Generates strongly-typed Rust code
4. Formats and lints the generated code

See [locale-dev README](../locale-dev/README.md) for details on the code generation pipeline.

## Updating to Latest CLDR

The library is automatically updated when new CLDR releases are available. To manually update:

```bash
# In the workspace root
cargo run -p locale-dev

# This will:
# 1. Check GitHub for the latest CLDR release
# 2. Generate updated code
# 3. Format and lint the generated code
```

## Licensing

This project respects and adheres to the licensing requirements of its source data:

- **Data Source**: All locale data is derived from the Unicode CLDR project and is subject to the **[Unicode License V3](https://www.unicode.org/license.txt)**.
- **Code Inspiration**: Architectural patterns inspired by [`num-format`](https://github.com/bcmyers/num-format), dual-licensed under **Apache-2.0** or **MIT**.
- **This Project**: Licensed under **[MIT License](../LICENSE-MIT)** or **[Apache-2.0 License](../LICENSE-APACHE)**.

## Contributing

Contributions are welcome! Since the core code is generated, most improvements should be directed toward:

- **locale-dev**: Improving code generation logic
- **locale-rs**: Adding new helper methods or improving documentation
- **Tests**: Expanding test coverage

If you find a missing locale or discrepancy with CLDR standards, please open an issue.

## See Also

- [locale-dev](../locale-dev/README.md) - Code generation tool
- [CLDR Project](https://cldr.unicode.org/) - Unicode locale data source
- [CLDR-JSON Repository](https://github.com/unicode-org/cldr-json) - GitHub source
- [num-format](https://github.com/bcmyers/num-format) - Inspiration for this library
