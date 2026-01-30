
# Locale

A comprehensive, strongly-typed Rust library for managing Unicode locales, built directly on the **CLDR (Common Locale Data Repository)** dataset.

This workspace contains two crates:

- **[locale-rs](./locale-rs/)** - The production library for locale management and formatting
- **[locale-dev](./locale-dev/)** - The code generation tool for updating locale data

## Project Philosophy

`Locale` is designed to be the foundational "source of truth" for locale identifiers in the Rust ecosystem. Rather than relying on hardcoded strings, this project leverages automated generation to stay perfectly in sync with the latest Unicode releases.

### Key Pillars

- **Authenticity**: Data is sourced directly from the [Unicode CLDR-JSON](https://github.com/unicode-org/cldr-json) repository
- **Safety**: Every locale identifier is a first-class citizen in a Rust `enum`, preventing typos and invalid locale errors at compile-time
- **Efficiency**: Zero-cost abstractions for locale identification and string conversion
- **Automation**: Automated code generation ensures the library stays in sync with Unicode standards
- **Inspiration**: Heavily inspired by the architectural patterns of the [num-format](https://github.com/bcmyers/num-format) crate

## Quick Start

### Using locale-rs

Add to your `Cargo.toml`:

```toml
[dependencies]
locale-rs = "0.2"

# With number formatting
locale-rs = { version = "0.2", features = ["nums"] }

# With all features
locale-rs = { version = "0.2", features = ["all"] }
```

Basic usage:

```rust
use locale_rs::Locale;

let locale = Locale::en_GB;
println!("{}", locale);  // "en-GB"

// Number formatting
use locale_rs::num_formats::ToFormattedString;
let num = 1234567;
println!("{}", num.to_formatted_string(&Locale::en));  // 1,234,567
println!("{}", num.to_formatted_string(&Locale::de));     // 1.234.567
```

### Updating Locale Data

To update to the latest CLDR release:

```bash
# In the workspace root
cargo run -p locale-dev

# This will:
# 1. Check GitHub for the latest CLDR-JSON release
# 2. Download or use cached data
# 3. Generate updated locale-rs code
# 4. Format and lint the generated code
```

## Workspace Structure

```
locale/
├── locale-rs/              # Production library
│   ├── src/
│   │   ├── lib.rs         # Public API
│   │   ├── locale.rs      # Auto-generated: Locale enum
│   │   ├── error.rs       # Error types
│   │   ├── num_formats.rs # Auto-generated: Number formatting
│   │   ├── currency_formats.rs  # Auto-generated: Currency patterns
│   │   └── datetime_formats.rs  # Auto-generated: DateTime data
│   ├── examples/          # Usage examples
│   ├── benches/           # Benchmarks
│   └── Cargo.toml
│
├── locale-dev/            # Code generation tool
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── download_latest.rs    # GitHub API & caching
│   │   ├── generate_locales.rs   # Locale enum generation
│   │   ├── generate_num_formats.rs       # Number format generation
│   │   ├── generate_currency_formatting.rs # Currency generation
│   │   ├── generate_datetime_formatting.rs # DateTime generation
│   │   ├── format.rs      # Code formatting
│   │   └── lib.rs         # Module exports
│   └── Cargo.toml
│
├── cache/                 # CLDR ZIP cache
├── Cargo.toml             # Workspace configuration
├── DOCUMENTATION.md       # Comprehensive documentation
└── README.md              # This file
```

## Features

### locale-rs Features

- **766 Unicode Locales** - Complete CLDR 48.1.0 coverage
- **Type-Safe Locales** - Compile-time validated enum variants
- **Number Formatting** - Locale-aware formatting with native digits
- **Currency Formatting** - ICU-compatible currency patterns
- **DateTime Formatting** - Localized month/weekday names
- **Flexible Parsing** - Parse with hyphens, underscores, or mixed case
- **Locale Negotiation** - Find best matching locale from available options
- **Fuzzy Suggestions** - Get suggestions for typos or unknown locales

### Supported Numbering Systems

- Latin (0-9)
- Arabic-Indic (٠-٩)
- Extended Arabic-Indic (۰-۹)
- Devanagari (०-९)
- Bengali (০-৯)
- Gujarati (૦-૯)
- Gurmukhi (੦-੯)
- Kannada (೦-೯)
- Malayalam (൦-൯)
- Oriya (୦-୯)
- Tamil (௦-௯)
- Telugu (౦-౯)
- Thai (๐-๙)
- Tibetan (༠-༩)
- And many more...

## Examples

### Basic Locale Operations

```rust
use locale_rs::Locale;
use std::str::FromStr;

// Direct enum access
let locale = Locale::en_GB;
assert_eq!(locale.as_str(), "en-GB");

// Parse from string
let locale = Locale::from_str("en-GB")?;

// Flexible parsing
let locale = Locale::from_flexible("en_gb")?;

// Extract subtags
assert_eq!(locale.language_code(), "en");
assert_eq!(locale.region_code(), Some("GB"));

// Fallback chain
assert_eq!(locale.fallback(), Some(Locale::en));
```

### Number Formatting

```rust
use locale_rs::Locale;
use locale_rs::num_formats::ToFormattedString;

let num = 1234567;

// Different locales, different formats
println!("{}", num.to_formatted_string(&Locale::en));     // 1,234,567
println!("{}", num.to_formatted_string(&Locale::de));     // 1.234.567
println!("{}", num.to_formatted_string(&Locale::fr));     // 1 234 567
println!("{}", num.to_formatted_string(&Locale::ar));     // ١٬٢٣٤٬٥٦٧
```

### Locale Negotiation

```rust
use locale_rs::Locale;

let user_preference = Locale::en_GB;
let available = vec![Locale::en, Locale::de, Locale::fr];

// Find best match (uses fallback chain)
if let Some(best) = user_preference.negotiate(&available) {
    println!("Using: {}", best);  // "en"
}
```

### Locale Suggestions

```rust
use locale_rs::Locale;

// Get suggestions for typos
let suggestions = Locale::suggest("en-gbb");
for locale in suggestions {
    println!("{}", locale);  // Suggests: en-GB, en, etc.
}
```

## Documentation

- **[locale-rs README](./locale-rs/README.md)** - Library usage guide
- **[locale-dev README](./locale-dev/README.md)** - Code generation tool guide
- **[DOCUMENTATION.md](./DOCUMENTATION.md)** - Comprehensive technical documentation

## Performance

- **Runtime**: Zero-cost abstractions; all operations are compile-time validated
- **Memory**: Locale enum variants are zero-sized types

## Licensing

This project respects and adheres to the licensing requirements of its source data:

- **Data Source**: All locale data is derived from the Unicode CLDR project and is subject to the **[Unicode License V3](https://www.unicode.org/license.txt)**
- **Code Inspiration**: Architectural patterns inspired by [`num-format`](https://github.com/bcmyers/num-format), dual-licensed under **Apache-2.0** or **MIT**
- **This Project**: Licensed under **[MIT License](./LICENSE-MIT)** or **[Apache-2.0 License](./LICENSE-APACHE)**

## Contributing

Contributions are welcome! Since the core code is generated, most improvements should be directed toward:

- **locale-dev**: Improving code generation logic and CLDR data extraction
- **locale-rs**: Adding new helper methods or improving documentation
- **Tests**: Expanding test coverage
- **Examples**: Adding more usage examples

If you find a missing locale or discrepancy with CLDR standards, please open an issue.

## Development

### Building

```bash
# Build all crates
cargo build --all

# Build with all features
cargo build --all --all-features
```

### Testing

```bash
# Run all tests
cargo test

# Run with all features
cargo test --features all

# Run specific test
cargo test --package locale-rs test_locales
```

### Benchmarking

```bash
# Run benchmarks
cargo bench -p locale-rs
```

### Code Generation

```bash
# Generate code from latest CLDR
cargo run -p locale-dev

# With verbose logging
RUST_LOG=debug cargo run -p locale-dev
```

## See Also

- [CLDR Project](https://cldr.unicode.org/) - Unicode locale data source
- [CLDR-JSON Repository](https://github.com/unicode-org/cldr-json) - GitHub source
- [num-format](https://github.com/bcmyers/num-format) - Inspiration for this library
- [BCP 47 Language Tags](https://tools.ietf.org/html/bcp47) - Locale identifier standard
