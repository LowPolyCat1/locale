
# Locale

A comprehensive, strongly-typed Rust library for managing Unicode locales, built directly on the **CLDR (Common Locale Data Repository)** dataset.

This crate provides a type-safe interface for locale identifiers, ensuring that your application remains compliant with international standards while benefiting from Rust's performance and safety guarantees.

## Project Philosophy

`Locale` is designed to be the foundational "source of truth" for locale identifiers in the Rust ecosystem. Rather than relying on hardcoded strings, this crate leverages automated generation to stay perfectly in sync with the latest Unicode releases.

### Key Pillars

* **Authenticity**: Data is sourced directly from the [Unicode CLDR-JSON](https://github.com/unicode-org/cldr-json) repository.
* **Safety**: Every locale identifier is a first-class citizen in a Rust `enum`, preventing typos and invalid locale errors at compile-time.
* **Efficiency**: Zero-cost abstractions for locale identification and string conversion.
* **Inspiration**: Heavily inspired by the architectural patterns of the [num-format](https://github.com/bcmyers/num-format) crate, focusing on high-performance formatting and developer ergonomic simplicity.

## Features

### Strongly-Typed Locales

Avoid string-parsing overhead. Use the Locale enum to ensure compile-time validity.

```rust
use locale_rs::Locale;

let my_locale = Locale::en_GB;
assert_eq!(my_locale.as_str(), "en-GB");
```

### Internationalized Number Formatting

Supports native numbering systems (e.g., Latin, Arabic-Indic, Devanagari) automatically based on the selected locale.

```shell
cargo run -p locale-rs --example arabic_nums --features nums
```

```shell
٠
١
٢
٣
٤
٥
٦
٧
٨
```

## Usage

```shell
git clone https://github.com/LowPolyCat1/locale
```

## Architecture

The project is structured as a dual-component workspace:

1. **Core Library (`locale`)**: The lightweight, production-ready crate containing the typed locale definitions and conversion traits.
2. **Developer Tool (`locale-dev`)**: An automated pipeline that fetches, caches, and parses the latest Unicode releases to generate the library's source code.

This structure ensures that the library remains updated without requiring manual maintenance of thousands of locale variants.

## Licensing

This project respects and adheres to the licensing requirements of its source data and inspirations:

* **Data Source**: All locale data is derived from the Unicode CLDR project and is subject to the **[Unicode License V3](https://www.unicode.org/license.txt)**.
* **Code Inspiration**: Architectural patterns and design philosophies are inspired by [`num-format`](<https://github.com/bcmyers/num-format>), which is dual-licensed under **Apache-2.0** or **MIT**.
* **This Project**: This library is licensed under the **[MIT License](./LICENSE-MIT)** or **[Apache2.0 License](./LICENSE-APACHE)**.

## Contribution

Contributions are welcome! If you find a missing locale or a discrepancy with the CLDR standards, please open an issue. Since the core code is generated, most improvements will be directed toward the logic in the `locale-dev` component.
