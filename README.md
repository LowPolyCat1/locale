
<div align="center">

# 🌐 Locale

A comprehensive, strongly-typed Rust library for managing Unicode locales, built directly on the **CLDR (Common Locale Data Repository)** dataset.

[![Rust](https://img.shields.io/badge/Rust-1f4068?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/locale-rs?style=for-the-badge&logo=rust&logoColor=white&color=1f4068)](https://crates.io/crates/locale-rs)
[![Docs.rs](https://img.shields.io/docsrs/locale-rs?style=for-the-badge&logo=docs.rs&logoColor=white&color=1f4068)](https://docs.rs/locale-rs)

[![CLDR](https://img.shields.io/badge/CLDR-48.1.0-162447?style=for-the-badge)](https://cldr.unicode.org/)
[![Crates.io License](https://img.shields.io/crates/l/locale-rs?style=for-the-badge&color=162447)](https://crates.io/crates/locale-rs)

[![Build Status](https://img.shields.io/github/actions/workflow/status/LowPolyCat1/locale/build.yml?style=for-the-badge&logo=github&label=Build&color=e43f5a)](https://github.com/LowPolyCat1/locale/actions)
![Tests Passing](https://img.shields.io/github/actions/workflow/status/LowPolyCat1/locale/build.yml?style=for-the-badge&label=tests&logo=github&color=e43f5a)
[![Crates.io Downloads](https://img.shields.io/crates/d/locale-rs?style=for-the-badge&color=e43f5a)](https://crates.io/crates/locale-rs)

</div>

---

This workspace contains two crates:

* **[locale-rs](./locale-rs/)** The production library for locale management and formatting.
* **[locale-dev](./locale-dev/)** The code generation tool for updating locale data.

## Project Philosophy

`Locale` is designed to be the foundational "source of truth" for locale identifiers in the Rust ecosystem. Rather than relying on hardcoded strings, this project leverages automated generation to stay perfectly in sync with the latest Unicode releases.

* **Authenticity:** Data is sourced directly from the official [Unicode CLDR-JSON](https://github.com/unicode-org/cldr-json) repository.
* **Safety:** Every locale identifier is a first-class citizen in a Rust `enum`, preventing typos and invalid locale errors at compile-time.
* **Efficiency:** Zero-cost abstractions for locale identification and string conversion.
* **Automation:** Automated code generation ensures the library stays in sync with Unicode standards.
* **Inspiration:** Heavily inspired by the architectural patterns of the [`num-format`](https://github.com/bcmyers/num-format) crate.

## Quick Start

### Using `locale-rs`

Add to your `Cargo.toml`:

```toml
[dependencies]
# Standard installation
locale-rs = "0.3"

# Or opt into specific features
locale-rs = { version = "0.3", features = ["nums"] }
locale-rs = { version = "0.3", features = ["all"] }

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
println!("{}", num.to_formatted_string(&Locale::de));  // 1.234.567

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
├── locale-rs/                 # Production library
│   ├── src/
│   │   ├── lib.rs             # Public API
│   │   ├── locale.rs          # Auto-generated: Locale enum
│   │   ├── error.rs           # Error types
│   │   ├── num_formats.rs     # Auto-generated: Number formatting
│   │   ├── currency_formats.rs# Auto-generated: Currency patterns
│   │   └── datetime_formats.rs# Auto-generated: DateTime data
│   ├── examples/              # Usage examples
│   ├── benches/               # Benchmarks
│   └── Cargo.toml
├── locale-dev/                # Code generation tool
│   ├── src/
│   │   ├── main.rs            # Entry point
│   │   ├── download_latest.rs # GitHub API & caching
│   │   ├── generate_locales.rs# Locale enum generation
│   │   └── ...
└── ...

```

## Features & Capabilities

### `locale-rs` Features

| Feature | Description |
| --- | --- |
| **766 Unicode Locales** | Complete CLDR 48.1.0 coverage out of the box. |
| **Type-Safe Locales** | Compile-time validated enum variants. |
| **Number Formatting** | Locale-aware formatting using native digits. |
| **Currency Formatting** | ICU-compatible currency patterns. |
| **DateTime Formatting** | Localized month and weekday names. |
| **Flexible Parsing** | Parse seamlessly with hyphens, underscores, or mixed cases. |
| **Locale Negotiation** | Find the best matching locale from available options using fallback chains. |
| **Fuzzy Suggestions** | Intelligently suggest corrections for typos or unknown locales. |

* Latin (0-9)
* Arabic-Indic (٠-٩)
* Extended Arabic-Indic (۰-۹)
* Devanagari (०-९)
* Bengali (০-৯)
* Gujarati (૦-૯)
* Gurmukhi (੦-੯)
* Kannada (೦-೯)
* Malayalam (൦-൯)
* Oriya (୦-୯)
* Tamil (௦-௯)
* Telugu (౦-౯)
* Thai (๐-๙)
* Tibetan (༠-༩)
* *And many more...*

## Deep-Dive Examples

### Basic Locale Operations

```rust
use locale_rs::Locale;
use std::str::FromStr;

// Direct enum access
let locale = Locale::en_GB;
assert_eq!(locale.as_str(), "en-GB");

// Parse from string & flexible alternatives
let locale = Locale::from_str("en-GB")?;
let locale = Locale::from_flexible("en_gb")?;

// Extract subtags & fallback chain
assert_eq!(locale.language_code(), "en");
assert_eq!(locale.region_code(), Some("GB"));
assert_eq!(locale.fallback(), Some(Locale::en));

```

### Locale Negotiation & Suggestions

```rust
use locale_rs::Locale;

// 1. Negotiation
let user_preference = Locale::en_GB;
let available = vec![Locale::en, Locale::de, Locale::fr];

if let Some(best) = user_preference.negotiate(&available) {
    println!("Using: {}", best);  // Falls back to "en"
}

// 2. Fuzzy Suggestions
let suggestions = Locale::suggest("en-gbb");
for locale in suggestions {
    println!("{}", locale);  // Suggests: en-GB, en, etc.
}

```

## Performance

* **Runtime:** Zero-cost abstractions; all data mapping operations are compile-time validated.
* **Memory:** Locale enum variants are completely zero-sized types.

## Licensing & Data Attribution

This project respects and adheres to the licensing requirements of its source data:

* **Data Source:** All locale data is derived from the Unicode CLDR project and is subject to the **[Unicode License V3](https://www.unicode.org/license.txt)**.
* **Code Inspiration:** Architectural patterns inspired by [`num-format`](https://github.com/bcmyers/num-format), dual-licensed under **Apache-2.0** or **MIT**.
* **This Project:** Dual-licensed under **[MIT License](https://www.google.com/search?q=./LICENSE-MIT)** or **[Apache-2.0 License](https://www.google.com/search?q=./LICENSE-APACHE)**.

## Contributing

Contributions are welcome! Since the core code is generated, most improvements should be directed toward:

* **`locale-dev`** — Improving code generation logic and CLDR data extraction.
* **`locale-rs`** — Adding new helper methods, testing runtime scenarios, or extending examples.
