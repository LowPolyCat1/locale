# Locale Project Documentation

A comprehensive, strongly-typed Rust library for managing Unicode locales, built directly on the **CLDR (Common Locale Data Repository)** dataset. This documentation covers both the core library (`locale-rs`) and the development tool (`locale-dev`).

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [locale-rs: Core Library](#locale-rs-core-library)
4. [locale-dev: Development Tool](#locale-dev-development-tool)
5. [API Reference](#api-reference)
6. [Usage Examples](#usage-examples)
7. [Development Workflow](#development-workflow)

---

## Project Overview

### Purpose

The Locale project provides a type-safe, compile-time validated interface for working with Unicode locales in Rust. Instead of relying on error-prone string-based locale identifiers, developers use strongly-typed Rust enums that prevent invalid locales at compile time.

### Key Features

- **766 Unicode Locales**: Complete coverage of CLDR 48.1.0 locales
- **Type Safety**: Locale identifiers are first-class Rust enums, preventing typos and invalid values
- **Zero-Cost Abstractions**: Compile-time locale validation with no runtime overhead
- **Automatic Updates**: Automated pipeline fetches and generates code from the latest CLDR releases
- **Comprehensive Formatting**: Support for number, currency, and datetime formatting
- **Native Numbering Systems**: Automatic support for Arabic-Indic, Devanagari, and other native digit systems

### Project Philosophy

The project is designed to be the foundational "source of truth" for locale identifiers in the Rust ecosystem. Rather than maintaining hardcoded locale lists, the codebase uses automated generation to stay perfectly in sync with Unicode standards.

---

## Architecture

### Workspace Structure

```
locale/
├── locale-rs/          # Production library (published to crates.io)
│   ├── src/
│   │   ├── lib.rs      # Public API exports
│   │   ├── locale.rs   # Auto-generated: Locale enum and core methods
│   │   ├── error.rs    # Error types
│   │   ├── num_formats.rs        # Auto-generated: Number formatting
│   │   ├── currency_formats.rs   # Auto-generated: Currency formatting
│   │   └── datetime_formats.rs   # Auto-generated: DateTime formatting
│   └── Cargo.toml
│
├── locale-dev/         # Development tool (not published)
│   ├── src/
│   │   ├── main.rs     # Entry point for code generation
│   │   ├── lib.rs      # Module exports
│   │   ├── error.rs    # Error types
│   │   ├── download_latest.rs    # GitHub API integration
│   │   ├── generate_locales.rs   # Locale enum generation
│   │   ├── generate_num_formats.rs       # Number format generation
│   │   ├── generate_datetime_formatting.rs # DateTime format generation
│   │   ├── generate_currency_formatting.rs # Currency format generation
│   │   ├── format.rs   # Code formatting and linting
│   │   └── test.rs     # Tests
│   └── Cargo.toml
│
├── cache/              # Cached CLDR ZIP files
├── Cargo.toml          # Workspace configuration
└── README.md
```

### Data Flow

```
GitHub (CLDR-JSON)
    ↓
download_latest.rs (fetches & caches)
    ↓
generate_locales.rs (creates Locale enum)
generate_num_formats.rs (creates number formatting)
generate_currency_formats.rs (creates currency formatting)
generate_datetime_formats.rs (creates datetime formatting)
    ↓
format.rs (runs cargo fmt & clippy)
    ↓
locale-rs/src/*.rs (generated code)
    ↓
Published to crates.io
```

---

## locale-rs: Core Library

### Module Overview

#### `lib.rs`

The public API entry point. Exports:
- `Locale` enum and `AVAILABLE_LOCALES` constant
- Feature-gated modules for optional functionality

```rust
pub mod error;
pub mod locale;
#[cfg(feature = "nums")]
pub mod num_formats;
#[cfg(feature = "currency")]
pub mod currency_formats;
#[cfg(feature = "datetime")]
pub mod datetime_formats;
```

#### `locale.rs` (Auto-generated)

**Purpose**: Defines the core `Locale` enum and provides locale manipulation methods.

**Key Components**:

1. **`Locale` Enum**: 766 variants representing all Unicode locales
   - Variants use underscores for hyphens (e.g., `en_GB` for "en-GB")
   - Rust keywords are escaped with trailing underscore (e.g., `as_`)

2. **`AVAILABLE_LOCALES` Constant**: Array of all 766 locale strings for iteration

3. **Core Methods**:
   - `as_str()`: Returns the string representation ("en-GB")
   - `fallback()`: Returns parent locale (e.g., `en_GB.fallback()` → `Some(en)`)
   - `language_code()`: Extracts language subtag ("en")
   - `region_code()`: Extracts region subtag (Some("GB"))
   - `from_flexible()`: Parses locale strings with flexible formatting
   - `negotiate()`: Finds best matching locale from available list
   - `suggest()`: Provides fuzzy-matched locale suggestions

4. **Trait Implementations**:
   - `FromStr`: Parse from strings
   - `TryFrom<&str>`: Fallible conversion
   - `From<Locale>` for `&'static str` and `String`
   - `Display`: Format as string
   - `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Example Generated Code**:
```rust
pub enum Locale {
    en,
    en_GB,
    de,
    // ... 763 more variants
}

impl Locale {
    pub fn as_str(&self) -> &'static str {
        match self {
            Locale::en => "en",
            Locale::en_GB => "en-GB",
            // ...
        }
    }
}
```

#### `error.rs`

Defines error types for locale operations:

```rust
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LocaleError {
    #[error("Unknown Error Occurred: '{0}'")]
    Unknown(String),
    #[error("Unknown locale identifier: '{0}'")]
    UnknownLocale(String),
}
```

#### `num_formats.rs` (Auto-generated)

**Purpose**: Provides locale-specific number formatting data and traits.

**Key Components**:

1. **Formatting Data Methods**:
   - `decimal_separator()`: Returns decimal point character ("." or ",")
   - `grouping_separator()`: Returns thousands separator ("," or " ")
   - `grouping_sizes()`: Returns array of grouping sizes (e.g., [3] for thousands)
   - `minus_sign()`: Returns minus sign character ("-" or "−")
   - `digits()`: Returns native digit characters (e.g., Arabic-Indic digits)

2. **`ToFormattedString` Trait**: Implemented for all numeric types
   - Provides `to_formatted_string(&self, locale: &Locale) -> String`
   - Handles grouping, decimal separators, and native digits

3. **Helper Functions**:
   - `translate_digits()`: Converts ASCII digits to native numbering system
   - `_format_int_str()`: Internal function for grouping integer portions

**Example**:
```rust
impl Locale {
    pub fn decimal_separator(&self) -> &'static str {
        match self {
            Locale::en => ".",
            Locale::de => ",",
            // ...
        }
    }
}

impl ToFormattedString for i32 {
    fn to_formatted_string(&self, locale: &Locale) -> String {
        // Formats with locale-specific separators and digits
    }
}
```

#### `currency_formats.rs` (Auto-generated)

**Purpose**: Provides locale-specific currency formatting patterns.

**Key Components**:

1. **Currency Pattern Methods**:
   - `currency_standard_pattern()`: Returns ICU number format pattern
   - `currency_accounting_pattern()`: Returns accounting format (for negative numbers)

2. **Pattern Format**: Uses ICU DecimalFormat syntax
   - `¤` = currency symbol
   - `#,##0.00` = number format
   - `\u{a0}` = non-breaking space
   - `;` = positive;negative pattern separator

**Example Patterns**:
```
"¤#,##0.00"           // $1,234.56 (US English)
"#,##0.00\u{a0}¤"    // 1.234,56 € (German)
"¤\u{a0}#,##0.00"    // $ 1,234.56 (French)
```

#### `datetime_formats.rs` (Auto-generated)

**Purpose**: Provides locale-specific datetime formatting data.

**Key Components**:

1. **`DateTime` Struct**: Represents a point in time
   ```rust
   pub struct DateTime {
       pub year: i32,
       pub month: u32,  // 1-12
       pub day: u32,    // 1-31
       pub hour: u32,   // 0-23
       pub minute: u32, // 0-59
       pub second: u32, // 0-59
   }
   ```

2. **Locale Data Methods**:
   - `months_wide()`: Full month names ("January", "Janvier", etc.)
   - `months_abbreviated()`: Short month names ("Jan", "Janv", etc.)
   - `weekdays_wide()`: Full weekday names
   - `weekdays_abbreviated()`: Short weekday names
   - `eras()`: Era names (e.g., "AD", "BC")
   - `date_format_pattern()`: Date formatting pattern
   - `time_format_pattern()`: Time formatting pattern
   - `datetime_format_pattern()`: Combined datetime pattern

### Features

The library uses Cargo features to control optional functionality:

```toml
[features]
rebuild = []              # Trigger code regeneration
strum = ["dep:strum", "dep:strum_macros"]  # Enum iteration
datetime = []             # DateTime formatting
nums = []                 # Number formatting
currency = ["nums"]       # Currency formatting (requires nums)
all = ["datetime", "nums", "strum", "currency"]
```

### Usage Patterns

#### Basic Locale Usage
```rust
use locale_rs::Locale;

let locale = Locale::en_GB;
assert_eq!(locale.as_str(), "en-GB");
assert_eq!(locale.language_code(), "en");
assert_eq!(locale.region_code(), Some("GB"));
```

#### Parsing Locales
```rust
use locale_rs::Locale;
use std::str::FromStr;

let locale = Locale::from_str("en-GB")?;
let locale = Locale::from_flexible("en_gb")?;  // Case-insensitive, flexible separators
```

#### Number Formatting
```rust
use locale_rs::Locale;
use locale_rs::num_formats::ToFormattedString;

let num = 1234567;
let en = Locale::en;
let de = Locale::de;

println!("{}", num.to_formatted_string(&en));  // 1,234,567
println!("{}", num.to_formatted_string(&de));     // 1.234.567
```

#### Locale Negotiation
```rust
use locale_rs::Locale;

let user_preference = Locale::en_GB;
let available = vec![Locale::en, Locale::de, Locale::fr];

if let Some(best) = user_preference.negotiate(&available) {
    println!("Using: {}", best);  // Prints: Using: en
}
```

---

## locale-dev: Development Tool

### Purpose

The `locale-dev` tool is a code generation pipeline that:
1. Fetches the latest CLDR data from GitHub
2. Parses locale definitions and formatting rules
3. Generates Rust code for the `locale-rs` library
4. Formats and lints the generated code

### Module Overview

#### `main.rs`

Entry point for the code generation pipeline. Orchestrates the workflow:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    match download_latest::get_latest_asset()? {
        Some(asset) => {
            generate_locales::run(...)?;
            generate_num_formats::run(...)?;
            generate_datetime_formatting::run(...)?;
            generate_currency_formatting::run(...)?;
            format::format_generated_code();
        }
        None => {
            tracing::info!("Local code is already up-to-date.");
        }
    }
    Ok(())
}
```

#### `download_latest.rs`

**Purpose**: Fetches CLDR data from GitHub and manages caching.

**Key Functions**:

1. **`get_latest_asset()`**: Main entry point
   - Queries GitHub API for latest CLDR-JSON release
   - Checks local cache before downloading
   - Returns `CldrAsset` with name and buffer

2. **Caching Strategy**:
   - Stores ZIP files in `cache/` directory
   - Avoids re-downloading if file already exists
   - Enables offline development

**Implementation Details**:
- Uses `reqwest` for HTTP requests
- Implements 300-second timeout for large downloads
- Sets user-agent to "rust-locale-gen"
- Parses GitHub API JSON response

#### `generate_locales.rs`

**Purpose**: Generates the `Locale` enum and core locale methods.

**Key Functions**:

1. **`run()`**: Main generation function
   - Extracts locale names from CLDR ZIP structure
   - Generates enum variants and match arms
   - Creates helper methods (fallback, language_code, region_code, etc.)

2. **Locale Extraction**:
   - Scans ZIP for directories matching `/main/{locale}/`
   - Extracts locale identifier from path
   - Sorts alphabetically for deterministic output

3. **Generated Code Sections**:

   a. **Enum Definition**:
   ```rust
   pub enum Locale {
       en,
       en_GB,
       de,
       // ...
   }
   ```

   b. **`as_str()` Method**: Maps enum variants to string representations

   c. **`fallback()` Method**: Implements locale fallback chain
   - Strips rightmost subtag (e.g., en_GB → en)
   - Returns None if no parent exists

   d. **`from_flexible()` Method**: Flexible string parsing
   - Accepts hyphens or underscores
   - Case-insensitive matching
   - Normalizes input before lookup

   e. **`negotiate()` Method**: Locale matching algorithm
   - Tries exact match first
   - Falls back through parent chain
   - Returns best available match

   f. **`suggest()` Method**: Fuzzy locale suggestions
   - Uses Levenshtein distance for similarity
   - Returns up to 5 suggestions
   - Filters by distance threshold (≤3)

   g. **`language_code()` & `region_code()` Methods**: Subtag extraction

4. **Trait Implementations**:
   - `FromStr`: Parses normalized strings
   - `TryFrom<&str>`: Fallible conversion
   - `From<Locale>` for string types
   - `Display`: Formats as string

5. **Helper Functions**:
   - `_levenshtein_distance()`: Calculates edit distance for suggestions

#### `generate_num_formats.rs`

**Purpose**: Generates number formatting data and traits.

**Key Functions**:

1. **`run()`**: Main generation function
   - Extracts numbering system definitions
   - Parses locale-specific number formats
   - Generates formatting methods and traits

2. **Numbering System Detection**:
   - Reads `numberingSystems.json` from CLDR
   - Maps system names to digit characters
   - Supports: Latin, Arabic-Indic, Devanagari, etc.

3. **Format Pattern Analysis**:
   - Parses ICU DecimalFormat patterns
   - Extracts grouping sizes from pattern
   - Handles multiple grouping levels (e.g., Indian: 2,2,3)

4. **Generated Methods**:
   - `decimal_separator()`: Decimal point character
   - `grouping_separator()`: Thousands separator
   - `grouping_sizes()`: Array of grouping sizes
   - `minus_sign()`: Negative number indicator
   - `digits()`: Native digit characters

5. **`ToFormattedString` Trait**:
   - Implemented for all integer types (i8-i128, u8-u128, isize, usize)
   - Implemented for floating-point types (f32, f64)
   - Handles special cases: NaN, Infinity, negative numbers

6. **Helper Functions**:
   - `_format_int_str()`: Applies grouping separators
   - `translate_digits()`: Converts ASCII to native digits
   - `detect_all_groupings()`: Extracts grouping sizes from patterns

#### `generate_currency_formatting.rs`

**Purpose**: Generates currency formatting patterns.

**Key Functions**:

1. **`run()`**: Main generation function
   - Extracts currency format patterns from CLDR
   - Generates pattern methods for each locale

2. **Pattern Methods**:
   - `currency_standard_pattern()`: Standard currency format
   - `currency_accounting_pattern()`: Accounting format (optional)

3. **Pattern Format**:
   - Uses ICU DecimalFormat syntax
   - `¤` placeholder for currency symbol
   - Supports positive and negative patterns separated by `;`

#### `generate_datetime_formatting.rs`

**Purpose**: Generates datetime formatting data.

**Key Functions**:

1. **`run()`**: Main generation function
   - Extracts month/weekday names
   - Extracts date/time format patterns
   - Generates formatting data methods

2. **Generated Data**:
   - Month names (wide and abbreviated)
   - Weekday names (wide and abbreviated)
   - Era names (AD, BC, etc.)
   - Date format patterns
   - Time format patterns
   - Combined datetime patterns

#### `format.rs`

**Purpose**: Formats and lints generated code.

**Key Functions**:

1. **`format_generated_code()`**: Post-generation cleanup
   - Runs `cargo fmt` on generated code
   - Runs `cargo clippy --fix` to apply suggestions
   - Ensures code quality and consistency

#### `lib.rs`

Module exports and shared utilities:

```rust
pub mod download_latest;
pub mod error;
pub mod format;
pub mod generate_currency_formatting;
pub mod generate_datetime_formatting;
pub mod generate_locales;
pub mod generate_num_formats;

pub fn sanitize_variant(name: &str) -> String {
    // Converts locale strings to valid Rust identifiers
    // Handles hyphens → underscores
    // Escapes Rust keywords with trailing underscore
}
```

**`sanitize_variant()` Function**:
- Replaces hyphens with underscores
- Escapes Rust keywords (as, if, fn, etc.)
- Ensures valid enum variant names

#### `error.rs`

Error types for the development tool:

```rust
#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Invalid header (expected {expected:?}, got {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("Missing attribute: {0}")]
    MissingAttribute(String),
}
```

### Running the Tool

```bash
# Generate code from latest CLDR release
cargo run -p locale-dev

# The tool will:
# 1. Check GitHub for latest CLDR-JSON release
# 2. Download or use cached ZIP file
# 3. Generate locale.rs, num_formats.rs, etc.
# 4. Format and lint the generated code
```

### Caching Behavior

- First run: Downloads CLDR ZIP (~100MB)
- Subsequent runs: Uses cached file if no newer release exists
- Cache location: `cache/cldr-{version}-json-full.zip`

---

## API Reference

### Core Types

#### `Locale` Enum

```rust
pub enum Locale {
    // 766 variants representing all Unicode locales
    en,
    en_GB,
    de,
    de_DE,
    // ... etc
}
```

#### `LocaleError`

```rust
pub enum LocaleError {
    Unknown(String),
    UnknownLocale(String),
}
```

#### `DateTime` (datetime feature)

```rust
pub struct DateTime {
    pub year: i32,
    pub month: u32,   // 1-12
    pub day: u32,     // 1-31
    pub hour: u32,    // 0-23
    pub minute: u32,  // 0-59
    pub second: u32,  // 0-59
}
```

### Core Methods

#### Locale Methods

| Method | Returns | Purpose |
|--------|---------|---------|
| `as_str()` | `&'static str` | Get string representation |
| `fallback()` | `Option<Locale>` | Get parent locale |
| `language_code()` | `&'static str` | Extract language subtag |
| `region_code()` | `Option<&'static str>` | Extract region subtag |
| `from_flexible(s)` | `Result<Locale, LocaleError>` | Parse with flexible formatting |
| `negotiate(available)` | `Option<Locale>` | Find best match from list |
| `suggest(input)` | `Vec<Locale>` | Get fuzzy suggestions |

#### Number Formatting Methods

| Method | Returns | Purpose |
|--------|---------|---------|
| `decimal_separator()` | `&'static str` | Get decimal point character |
| `grouping_separator()` | `&'static str` | Get thousands separator |
| `grouping_sizes()` | `&'static [usize]` | Get grouping size array |
| `minus_sign()` | `&'static str` | Get negative sign |
| `digits()` | `Option<[char; 10]>` | Get native digit characters |

#### Currency Formatting Methods

| Method | Returns | Purpose |
|--------|---------|---------|
| `currency_standard_pattern()` | `&'static str` | Get standard currency pattern |
| `currency_accounting_pattern()` | `&'static str` | Get accounting pattern |

#### DateTime Formatting Methods

| Method | Returns | Purpose |
|--------|---------|---------|
| `months_wide()` | `&'static [&'static str]` | Get full month names |
| `months_abbreviated()` | `&'static [&'static str]` | Get short month names |
| `weekdays_wide()` | `&'static [&'static str]` | Get full weekday names |
| `weekdays_abbreviated()` | `&'static [&'static str]` | Get short weekday names |

### Traits

#### `ToFormattedString`

```rust
pub trait ToFormattedString {
    fn to_formatted_string(&self, locale: &Locale) -> String;
}
```

Implemented for: i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64

#### Standard Traits

- `FromStr`: Parse from strings
- `TryFrom<&str>`: Fallible conversion
- `From<Locale>` for `&'static str` and `String`
- `Display`: Format as string
- `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

---

## Usage Examples

### Basic Locale Operations

```rust
use locale_rs::Locale;
use std::str::FromStr;

// Direct enum access
let locale = Locale::en_GB;
println!("{}", locale);  // "en-GB"

// Parsing from strings
let locale = Locale::from_str("en-GB")?;
let locale = Locale::from_flexible("en_gb")?;  // Case-insensitive

// Extracting subtags
assert_eq!(locale.language_code(), "en");
assert_eq!(locale.region_code(), Some("GB"));

// Fallback chain
assert_eq!(Locale::en_GB.fallback(), Some(Locale::en));
assert_eq!(Locale::en.fallback(), None);
```

### Number Formatting

```rust
use locale_rs::Locale;
use locale_rs::num_formats::ToFormattedString;

let num = 1234567;

// US English: 1,234,567
println!("{}", num.to_formatted_string(&Locale::en));

// German: 1.234.567
println!("{}", num.to_formatted_string(&Locale::de));

// French: 1 234 567
println!("{}", num.to_formatted_string(&Locale::fr));

// Arabic with native digits
println!("{}", num.to_formatted_string(&Locale::ar));
```

### Locale Negotiation

```rust
use locale_rs::Locale;

let user_locales = vec![Locale::en_GB, Locale::en];
let available = vec![Locale::en, Locale::de, Locale::fr];

// Find best match
if let Some(best) = Locale::en_GB.negotiate(&available) {
    println!("Using: {}", best);  // "en"
}
```

### Locale Suggestions

```rust
use locale_rs::Locale;

// Typo correction
let suggestions = Locale::suggest("en-gbb");
for locale in suggestions {
    println!("{}", locale);  // Suggests: en-GB, en, etc.
}
```

### Currency Formatting

```rust
use locale_rs::Locale;

let locale = Locale::en;
let pattern = locale.currency_standard_pattern();
println!("{}", pattern);  // "¤#,##0.00"

// Use with a currency formatting library
```

### DateTime Formatting

```rust
use locale_rs::Locale;

let locale = Locale::de;
let months = locale.months_wide();
println!("{}", months[0]);  // "Januar"

let weekdays = locale.weekdays_abbreviated();
println!("{}", weekdays[0]);  // "Mo"
```

---

## Development Workflow

### Adding a New Locale

Locales are automatically generated from CLDR data. To add a new locale:

1. **Wait for CLDR Release**: New locales are added to CLDR periodically
2. **Run Code Generation**: `cargo run -p locale-dev`
3. **Commit Changes**: The generated code will be updated automatically

### Updating CLDR Data

```bash
# Remove cached CLDR file to force re-download
rm cache/cldr-*.zip

# Run code generation
cargo run -p locale-dev

# Commit the updated generated code
git add locale-rs/src/*.rs
git commit -m "Update to latest CLDR release"
```

### Testing

```bash
# Run all tests
cargo test

# Run with all features
cargo test --features all

# Run specific test
cargo test --package locale-rs test_locales

# Run benchmarks
cargo bench -p locale-rs
```

### Code Generation Process

1. **Download Phase** (`download_latest.rs`):
   - Queries GitHub API for latest CLDR-JSON release
   - Downloads ZIP file (or uses cache)
   - Returns buffer and asset name

2. **Generation Phase** (multiple generators):
   - Parse CLDR JSON structure
   - Extract locale definitions and formatting rules
   - Generate Rust code with match arms for each locale

3. **Formatting Phase** (`format.rs`):
   - Run `cargo fmt` for consistent style
   - Run `cargo clippy --fix` for linting
   - Ensure generated code quality

### Performance Considerations

- **Compile Time**: Generated code is large (~50K lines) but compiles efficiently
- **Binary Size**: Minimal impact due to static data
- **Runtime**: Zero-cost abstractions; all formatting is compile-time validated
- **Memory**: Locale enum variants are zero-sized types

### Extending the Library

To add new formatting capabilities:

1. **Add Generator Module**: Create `locale-dev/src/generate_*.rs`
2. **Implement Generation Logic**: Parse CLDR data and generate Rust code
3. **Update Main**: Add call to new generator in `locale-dev/src/main.rs`
4. **Add Public Module**: Export in `locale-rs/src/lib.rs`
5. **Add Feature Flag**: Control with Cargo feature if optional

---

## Licensing

This project respects and adheres to the licensing requirements of its source data:

- **Data Source**: All locale data is derived from the Unicode CLDR project and is subject to the [Unicode License V3](https://www.unicode.org/license.txt)
- **Code Inspiration**: Architectural patterns inspired by [`num-format`](https://github.com/bcmyers/num-format), dual-licensed under Apache-2.0 or MIT
- **This Project**: Licensed under [MIT License](./LICENSE-MIT) or [Apache-2.0 License](./LICENSE-APACHE)

---

## Contributing

Contributions are welcome! Since the core code is generated, most improvements should be directed toward:

- **locale-dev**: Improving code generation logic
- **locale-rs**: Adding new formatting capabilities or helper methods
- **Documentation**: Improving guides and examples

If you find a missing locale or discrepancy with CLDR standards, please open an issue.

---

## References

- [Unicode CLDR Project](https://cldr.unicode.org/)
- [CLDR-JSON Repository](https://github.com/unicode-org/cldr-json)
- [ICU DecimalFormat Documentation](https://unicode-org.github.io/icu/userguide/format_parse/numbers/decimal.html)
- [BCP 47 Language Tags](https://tools.ietf.org/html/bcp47)
