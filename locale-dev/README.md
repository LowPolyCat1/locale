# locale-dev

A code generation tool that automatically generates the `locale-rs` library from Unicode CLDR data.

## Overview

`locale-dev` is an internal development tool that:

1. **Fetches** the latest CLDR (Common Locale Data Repository) data from GitHub
2. **Parses** locale definitions, number formats, currency patterns, and datetime data
3. **Generates** strongly-typed Rust code for the `locale-rs` library
4. **Formats** and lints the generated code using `cargo fmt` and `cargo clippy`

This ensures that `locale-rs` stays perfectly in sync with the latest Unicode standards without manual maintenance.

## Quick Start

### Prerequisites

- Rust 1.70+
- Internet connection (for GitHub API access)
- ~100MB disk space (for CLDR ZIP cache)

### Running Code Generation

```bash
# Generate code from the latest CLDR release
cargo run -p locale-dev

# The tool will:
# 1. Check GitHub for the latest CLDR-JSON release
# 2. Download or use cached ZIP file
# 3. Generate locale.rs, num_formats.rs, currency_formats.rs, datetime_formats.rs
# 4. Format and lint the generated code
```

### Output

Generated files are written to:
- `locale-rs/src/locale.rs` - Locale enum and core methods
- `locale-rs/src/num_formats.rs` - Number formatting data and traits
- `locale-rs/src/currency_formats.rs` - Currency formatting patterns
- `locale-rs/src/datetime_formats.rs` - DateTime formatting data

## Architecture

### Module Structure

```
locale-dev/src/
├── main.rs                          # Entry point, orchestrates pipeline
├── lib.rs                           # Module exports and utilities
├── error.rs                         # Error types
├── download_latest.rs               # GitHub API integration & caching
├── generate_locales.rs              # Generates Locale enum
├── generate_num_formats.rs          # Generates number formatting
├── generate_currency_formatting.rs  # Generates currency patterns
├── generate_datetime_formatting.rs  # Generates datetime data
├── format.rs                        # Code formatting & linting
└── test.rs                          # Tests
```

### Data Flow

```
GitHub (CLDR-JSON Release)
    ↓
download_latest::get_latest_asset()
    ↓ (caches in cache/ directory)
    ↓
generate_locales::run()
generate_num_formats::run()
generate_currency_formatting::run()
generate_datetime_formatting::run()
    ↓
format::format_generated_code()
    ↓ (cargo fmt + cargo clippy)
    ↓
locale-rs/src/*.rs (updated)
```

## Module Documentation

### `download_latest.rs`

Fetches CLDR data from GitHub and manages local caching.

**Key Function**: `get_latest_asset() -> Result<Option<CldrAsset>>`

- Queries GitHub API for the latest CLDR-JSON release
- Downloads the `cldr-{version}-json-full.zip` file
- Caches in `cache/` directory to avoid re-downloading
- Returns `None` if local cache is already up-to-date

**Caching Strategy**:
- First run: Downloads ~100MB ZIP file
- Subsequent runs: Uses cached file if no newer release exists
- To force re-download: `rm cache/cldr-*.zip`

### `generate_locales.rs`

Generates the `Locale` enum and core locale manipulation methods.

**Key Function**: `run(zip_buffer, asset_name, output_path) -> Result<()>`

**Generated Code Includes**:

1. **Locale Enum** (766 variants)
   ```rust
   pub enum Locale {
   }
   ```

2. **Core Methods**:
   - `as_str()` - String representation
   - `fallback()` - Parent locale in fallback chain
   - `language_code()` - Extract language subtag
   - `region_code()` - Extract region subtag
   - `from_flexible()` - Parse with flexible formatting
   - `negotiate()` - Find best match from available list
   - `suggest()` - Fuzzy locale suggestions

3. **Trait Implementations**:
   - `FromStr` - Parse from strings
   - `TryFrom<&str>` - Fallible conversion
   - `From<Locale>` for string types
   - `Display` - Format as string

**Locale Extraction**:
- Scans CLDR ZIP for directories matching `/main/{locale}/`
- Extracts locale identifier from path
- Sorts alphabetically for deterministic output

**Fallback Chain**:
- Automatically detects parent locales
- Example: `en_GB` → `en` → `None`
- Used for locale negotiation

### `generate_num_formats.rs`

Generates number formatting data and the `ToFormattedString` trait.

**Key Function**: `run(zip_buffer, asset_name, output_path) -> Result<()>`

**Generated Code Includes**:

1. **Formatting Data Methods**:
   - `decimal_separator()` - Decimal point character ("." or ",")
   - `grouping_separator()` - Thousands separator ("," or " ")
   - `grouping_sizes()` - Array of grouping sizes
   - `minus_sign()` - Negative sign character
   - `digits()` - Native digit characters (e.g., Arabic-Indic)

2. **`ToFormattedString` Trait**:
   - Implemented for all integer types (i8-i128, u8-u128, isize, usize)
   - Implemented for floating-point types (f32, f64)
   - Handles special cases: NaN, Infinity, negative numbers

3. **Helper Functions**:
   - `translate_digits()` - Convert ASCII to native digits
   - `_format_int_str()` - Apply grouping separators

**Numbering System Support**:
- Reads `numberingSystems.json` from CLDR
- Supports: Latin, Arabic-Indic, Devanagari, Bengali, etc.
- Automatically detects native digit characters

**Grouping Analysis**:
- Parses ICU DecimalFormat patterns
- Extracts grouping sizes (e.g., [3] for thousands, [2,2,3] for Indian)
- Handles multiple grouping levels

### `generate_currency_formatting.rs`

Generates currency formatting patterns for each locale.

**Key Function**: `run(zip_buffer, asset_name, output_path) -> Result<()>`

**Generated Code Includes**:

1. **Pattern Methods**:
   - `currency_standard_pattern()` - Standard currency format
   - `currency_accounting_pattern()` - Accounting format (optional)

2. **Pattern Format** (ICU DecimalFormat syntax):
   - `¤` = currency symbol placeholder
   - `#,##0.00` = number format
   - `\u{a0}` = non-breaking space
   - `;` = positive;negative pattern separator

**Example Patterns**:
```
"¤#,##0.00"           // $1,234.56 (US English)
"#,##0.00\u{a0}¤"    // 1.234,56 € (German)
"¤\u{a0}#,##0.00"    // $ 1,234.56 (French)
```

### `generate_datetime_formatting.rs`

Generates datetime formatting data for each locale.

**Key Function**: `run(zip_buffer, asset_name, output_path) -> Result<()>`

**Generated Code Includes**:

1. **DateTime Struct**:
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

2. **Locale Data Methods**:
   - `months_wide()` - Full month names
   - `months_abbreviated()` - Short month names
   - `weekdays_wide()` - Full weekday names
   - `weekdays_abbreviated()` - Short weekday names
   - `eras()` - Era names (AD, BC, etc.)
   - `date_format_pattern()` - Date formatting pattern
   - `time_format_pattern()` - Time formatting pattern
   - `datetime_format_pattern()` - Combined datetime pattern

### `format.rs`

Post-generation code formatting and linting.

**Key Function**: `format_generated_code()`

- Runs `cargo fmt -p locale-rs` for consistent style
- Runs `cargo clippy -p locale-rs --fix` for linting
- Ensures generated code quality and consistency

### `lib.rs`

Module exports and shared utilities.

**Key Function**: `sanitize_variant(name: &str) -> String`

- Converts locale strings to valid Rust identifiers
- Replaces hyphens with underscores (e.g., "en-GB" → "en_GB")
- Escapes Rust keywords with trailing underscore (e.g., "as" → "as_")

**Rust Keywords Handled**:
```
as, break, const, continue, crate, else, enum, extern, false, fn, for,
if, impl, in, let, loop, match, mod, move, mut, pub, ref, return,
self, Self, static, struct, super, trait, true, type, unsafe, use, where,
while, async, await, dyn, abstract, become, box, do, final, macro,
override, priv, typeof, unsized, virtual, yield, try
```

## Usage Examples

### Basic Code Generation

```bash
# Generate from latest CLDR release
cargo run -p locale-dev

# Output:
# Checking GitHub for the latest CLDR asset...
# Using cached file: cache/cldr-48.1.0-json-full.zip
# generating locales
# Generated 766 locales.
# Refining generated code in locale-rs...
# Successfully formatted locale-rs.
# Clippy checks passed/fixed for locale-rs.
```

### Force Re-download

```bash
# Remove cached file
rm cache/cldr-*.zip

# Run generation (will download fresh copy)
cargo run -p locale-dev
```

### Check for Updates

```bash
# The tool automatically checks GitHub for newer releases
# If local cache is up-to-date, it will report:
# "Local code is already up-to-date. No action needed."
```

## Development

### Running Tests

```bash
# Run all tests
cargo test -p locale-dev

# Run with output
cargo test -p locale-dev -- --nocapture
```

### Debugging

Enable verbose logging:

```bash
# Set RUST_LOG environment variable
RUST_LOG=debug cargo run -p locale-dev
```

The tool uses `tracing` for structured logging:
- `info!()` - General progress messages
- `error!()` - Error conditions

### Adding New Generators

To add a new code generator:

1. **Create Module**: `locale-dev/src/generate_*.rs`
2. **Implement Function**: `pub fn run(zip_buffer, asset_name, output_path) -> Result<()>`
3. **Update Main**: Add call in `locale-dev/src/main.rs`
4. **Export Module**: Add to `locale-dev/src/lib.rs`
5. **Add Public Module**: Export in `locale-rs/src/lib.rs`

Example:

```rust
// locale-dev/src/generate_custom.rs
pub fn run(
    zip_buffer: Vec<u8>,
    asset_name: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLDR data
    // Generate Rust code
    // Write to output_path
    Ok(())
}
```

## Performance

### Generation Time

- **Download**: 10-30 seconds (first run, depends on network)
- **Parsing**: 5-10 seconds
- **Code Generation**: 2-5 seconds
- **Formatting**: 10-20 seconds
- **Total**: ~30-60 seconds (first run), ~20-40 seconds (cached)

### Output Size

- `locale.rs`: ~50KB (766 locale variants)
- `num_formats.rs`: ~150KB (formatting data)
- `currency_formats.rs`: ~200KB (currency patterns)
- `datetime_formats.rs`: ~300KB (datetime data)
- **Total**: ~700KB of generated code

### Memory Usage

- Peak memory during generation: ~500MB (ZIP parsing)
- Generated binary impact: Minimal (static data)

## Troubleshooting

### Network Issues

**Problem**: "Failed to connect to GitHub"

**Solution**:
```bash
# Check internet connection
ping api.github.com

# Use cached file if available
# The tool will use cache/cldr-*.zip if it exists
```

### Timeout Issues

**Problem**: "Request timeout"

**Solution**:
- The tool has a 300-second timeout for downloads
- For slow connections, manually download the ZIP:
  ```bash
  # Download from: https://github.com/unicode-org/cldr-json/releases
  # Place in: cache/cldr-{version}-json-full.zip
  ```

### Formatting Errors

**Problem**: "Cargo fmt encountered errors"

**Solution**:
```bash
# Check if cargo fmt is installed
cargo fmt --version

# Update Rust
rustup update
```

### Clippy Issues

**Problem**: "Clippy found issues that require manual attention"

**Solution**:
- Review the generated code for warnings
- Some warnings may require manual fixes
- Check `locale-rs/src/*.rs` for issues

## Contributing

Improvements to the code generation pipeline are welcome! Areas for contribution:

- **Performance**: Optimize parsing and generation
- **Accuracy**: Improve CLDR data extraction
- **Features**: Add new formatting capabilities
- **Testing**: Expand test coverage

## Dependencies

- `reqwest` - HTTP client for GitHub API
- `zip` - ZIP file handling
- `serde` & `serde_json` - JSON parsing
- `quote` & `proc-macro2` - Code generation
- `thiserror` - Error handling
- `tracing` & `tracing-subscriber` - Logging

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## See Also

- [locale-rs](../locale-rs/README.md) - The generated library
- [CLDR Project](https://cldr.unicode.org/) - Unicode locale data source
- [CLDR-JSON Repository](https://github.com/unicode-org/cldr-json) - GitHub source
