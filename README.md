
<div align="center">

# 🌐 Locale

A comprehensive, strongly-typed Rust library for managing Unicode locales, built directly on the **CLDR (Common Locale Data Repository)** dataset.

[![Rust](https://img.shields.io/badge/Rust-1f4068?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/locale-rs?style=for-the-badge&logo=rust&logoColor=white&color=1f4068)](https://crates.io/crates/locale-rs)
[![Docs.rs](https://img.shields.io/docsrs/locale-rs?style=for-the-badge&logo=docs.rs&logoColor=white&color=1f4068)](https://docs.rs/locale-rs)

<!-- gen:
[![CLDR](https://img.shields.io/badge/CLDR-{{cldr_version}}-162447?style=for-the-badge)](https://cldr.unicode.org/)
[![Crates.io License](https://img.shields.io/crates/l/locale-rs?style=for-the-badge&color=162447)](https://crates.io/crates/locale-rs)
-->
[![CLDR](https://img.shields.io/badge/CLDR-48.2.2-162447?style=for-the-badge)](https://cldr.unicode.org/)
[![Crates.io License](https://img.shields.io/crates/l/locale-rs?style=for-the-badge&color=162447)](https://crates.io/crates/locale-rs)
<!-- /gen -->

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

<!-- gen:
```toml
[dependencies]
# Standard installation
locale-rs = "{{crate_version_req}}"

# Or opt into specific features
locale-rs = { version = "{{crate_version_req}}", features = ["nums"] }
locale-rs = { version = "{{crate_version_req}}", features = ["all"] }
```
-->
```toml
[dependencies]
# Standard installation
locale-rs = "0.4.0-rc.1"

# Or opt into specific features
locale-rs = { version = "0.4.0-rc.1", features = ["nums"] }
locale-rs = { version = "0.4.0-rc.1", features = ["all"] }
```
<!-- /gen -->

Available Cargo features (none are enabled by default):

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

Basic usage:

```rust
use locale_rs::Locale;
use locale_rs::nums::ToFormattedString;

let locale = Locale::en_GB;
println!("{locale}");  // "en-GB"

let num = 1234567;
println!("{}", num.to_formatted_string(&Locale::en));  // 1,234,567
println!("{}", num.to_formatted_string(&Locale::de));  // 1.234.567
```

See the [locale-rs README](./locale-rs/README.md) for currency and date formatting.

### Updating Locale Data

To update to the latest CLDR release:

```bash
# In the workspace root
cargo run -p locale-dev

# This will:
# 1. Check GitHub for the latest CLDR-JSON release
# 2. Download it into ./cache (or reuse a cached copy)
# 3. Regenerate the data tables in locale-rs/src/data/ and rustfmt them
# 4. Bump the locale-rs version and sync the READMEs

# Regenerate from a local archive without touching versions:
cargo run -p locale-dev -- --archive path/to/cldr-48.2.2-json-full.zip
```

## Workspace Structure

```
locale/
├── locale-rs/                  # Production library
│   ├── src/
│   │   ├── lib.rs              # Public API
│   │   ├── locale.rs           # Locale parsing, fallback, negotiation
│   │   ├── nums.rs             # Number formatting
│   │   ├── currency.rs         # Currency formatting
│   │   ├── datetime.rs         # Date and time formatting
│   │   ├── format.rs           # Shared allocation-free writers
│   │   ├── error.rs            # Error types
│   │   └── data/               # Generated CLDR tables (do not edit)
│   │       ├── mod.rs          # Handwritten: shared data types
│   │       ├── locales.rs      # Locale enum, lookup map, parents
│   │       ├── numbers.rs      # Number symbols
│   │       ├── dates.rs        # Calendar names, parsed patterns
│   │       └── currency.rs     # Currency patterns and symbols
│   ├── examples/               # Usage examples
│   ├── benches/                # Benchmarks
│   └── Cargo.toml
├── locale-dev/                 # Code generation tool
│   ├── src/
│   │   ├── main.rs             # CLI entry point
│   │   ├── cldr.rs             # Reads a CLDR archive into a typed model
│   │   ├── patterns.rs         # Number, currency and date pattern parsers
│   │   ├── emit/               # One emitter per data module
│   │   ├── download_latest.rs  # GitHub API & caching
│   │   └── ...
└── ...
```

## Features & Capabilities

### `locale-rs` Features

| Feature | Description |
| --- | --- |
| **<!-- gen:{{locale_count}} -->766<!-- /gen --> Unicode Locales** | Complete CLDR <!-- gen:{{cldr_version}} -->48.2.2<!-- /gen --> coverage out of the box. |
| **Type-Safe Locales** | Compile-time validated enum variants. |
| **Number Formatting** | Locale-aware formatting using native digits. |
| **Currency Formatting** | CLDR currency patterns, symbols and fraction digits for any currency. |
| **DateTime Formatting** | Localized names with the medium date and time patterns. |
| **CLDR Inheritance** | Fallback chains follow CLDR parent locales. |
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
let locale = Locale::from_str("en-GB").unwrap();
let locale = Locale::from_flexible("en_gb").unwrap();

// Extract subtags & fallback chain (CLDR parent locales)
assert_eq!(locale.language_code(), "en");
assert_eq!(locale.region_code(), Some("GB"));
assert_eq!(locale.fallback(), Some(Locale::en_001));
assert_eq!(Locale::en_001.fallback(), Some(Locale::en));

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

* **Runtime:** Locale data is a direct array index; `FromStr` uses a perfect hash map.
* **Memory:** `Locale` is a two-byte `Copy` enum, and identical CLDR values are stored once.
* **Allocation-free:** Formatters implement `Display` and write straight into any buffer.

## Licensing & Data Attribution

This project respects and adheres to the licensing requirements of its source data:

* **Data Source:** All locale data is derived from the Unicode CLDR project and is subject to the **[Unicode License V3](https://www.unicode.org/license.txt)**.
* **Code Inspiration:** Architectural patterns inspired by [`num-format`](https://github.com/bcmyers/num-format), dual-licensed under **Apache-2.0** or **MIT**.
* **This Project:** Dual-licensed under **[MIT License](./LICENSE-MIT)** or **[Apache-2.0 License](./LICENSE-APACHE)**.

## Contributing

Contributions are welcome! Only the data tables in `locale-rs/src/data/` are generated; everything else is handwritten. Improvements are usually directed toward:

* **`locale-dev`** — Improving code generation logic and CLDR data extraction.
* **`locale-rs`** — Adding new helper methods, testing runtime scenarios, or extending examples.
