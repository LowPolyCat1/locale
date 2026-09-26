# locale-dev

The code generator that produces the CLDR data tables of `locale-rs`.

## Overview

`locale-dev` is an internal development tool that:

1. **Fetches** the latest CLDR-JSON release from GitHub (or reads a local archive)
2. **Reads** the archive once into a typed model (`cldr.rs`)
3. **Parses** number, currency and date patterns at generation time (`patterns.rs`)
4. **Emits** the data modules in `locale-rs/src/data/` (`emit/`) and formats them with rustfmt

Only data is generated. All formatting logic in `locale-rs` is handwritten Rust that is reviewed, linted and tested like any other code.

## Quick Start

### Prerequisites

- The Rust toolchain from `rust-version` in `locale-rs/Cargo.toml`, with `rustfmt`
- Internet connection for the GitHub API, unless you use `--archive`

### Commands

```bash
# Update to the latest CLDR release, if it is newer than the recorded one:
# downloads into ./cache, regenerates the data, makes a breaking version bump
# if the data changed, records the new CLDR version, writes the pull request
# description to target/cldr-bump-pr.md and syncs the READMEs.
cargo run -p locale-dev

# Regenerate from a local cldr-json archive. Versions stay untouched, which is
# what you want after changing the generator itself, or when offline. If the
# output changed, a warning says the next release must be breaking.
cargo run -p locale-dev -- --archive cache/cldr-48.2.2-json-full.zip

# Sync or check the generated README sections.
cargo run -p locale-dev -- readme
cargo run -p locale-dev -- readme --check
```

A local archive needs the layout of the `cldr-json` release assets: `cldr-core/`, `cldr-numbers-full/` and `cldr-dates-full/` at the top level.

## Release Policy

Every change to the generated data is a breaking release (`0.4.2` -> `0.5.0`, `1.2.3` -> `2.0.0`, `0.5.0-rc.1` -> `0.5.0-rc.2`). `policy.rs` compares the generated files before and after a regeneration:

- **Added or removed locales** change the exhaustive `Locale` enum, so downstream `match`es stop compiling. They are listed in the log and the pull request.
- **Changed symbols, patterns or `CLDR_VERSION`** compile fine but change output, which downstream tests and users may rely on.

Either way, downstream code only sees the change after an explicit upgrade, never through a plain `cargo update`, and the compiler points at every exhaustive `match` to revisit. Since `CLDR_VERSION` is part of the generated data, every CLDR update is a breaking release.

## Architecture

```
locale-dev/src/
├── main.rs              # CLI: update, --archive, readme
├── lib.rs               # generate(): runs every emitter, then rustfmt
├── cldr.rs              # Reads the archive once into the `Cldr` model
├── patterns.rs          # Parsers for number, currency and date patterns
├── emit/
│   ├── mod.rs           # `Pool` (deduplication) and `RustFile` (output)
│   ├── locales.rs       # -> data/locales.rs
│   ├── numbers.rs       # -> data/numbers.rs
│   ├── dates.rs         # -> data/dates.rs
│   └── currency.rs      # -> data/currency.rs
├── format.rs            # rustfmt over the generated files
├── download_latest.rs   # GitHub API and download cache
├── policy.rs            # Release policy: detects data changes
├── version.rs           # CLDR and crate version bookkeeping
├── readme.rs            # Generated README sections
├── error.rs             # `Error` type
└── test.rs              # Integration tests against synthesized archives
```

### The model (`cldr.rs`)

`Cldr::from_zip` opens the archive once and extracts everything `locale-rs` needs into plain structs: per locale its number symbols and patterns, Gregorian names and medium patterns, currency pattern, default currency and currency symbols, and its parent locale; plus the fraction digits of every currency. No emitter ever sees JSON.

CLDR rules resolved here:

- **Parent locales** come from `parentLocales.json` (`en-IN` inherits from `en-001`), then the `nonlikelyScript` rule (`sr-Latn` inherits from root because Latin is not Serbian's likely script), then subtag truncation. Ancestors missing from the archive are skipped.
- **Default currency** is the tender of the locale's own region, else of its language's likely region (`de` is `DE`). Macro-regions such as `419` have no currency and get `USD`.

### The emitters (`emit/`)

Each emitter turns the model into one Rust file of `static` tables indexed by `Locale as usize`:

| Output | Contents |
| --- | --- |
| `data/locales.rs` | `CLDR_VERSION`, `AVAILABLE_LOCALES`, the `Locale` enum, a `phf` map for parsing and the parent table |
| `data/numbers.rs` | `NumberSymbols` per locale, with native digit tables |
| `data/dates.rs` | Month and weekday names, `DatePattern`s pre-parsed into `DatePart`s |
| `data/currency.rs` | Pre-parsed `CurrencyPattern`s, default currencies, symbol overrides, fraction digits |

Identical values (a month list, a symbol set, a pattern) are stored once through `Pool`. Currency symbols are stored as overrides of the parent locale, and `locale-rs` walks the fallback chain at runtime; the emitter checks that this resolves every symbol exactly as CLDR lists it.

Items are built with `quote!` and laid out with `prettyplease`; per-locale tables are written one entry per line with the locale as a comment, so a CLDR update produces a reviewable diff. rustfmt runs last, so the output is identical to `cargo fmt`.

The `phf` map is built here with `phf_codegen`, so users of `locale-rs` do not compile the `phf` proc-macro.

### Handwritten types

The types the tables are made of live in `locale-rs`, not here: `Grouping` and `NumberSymbols` in `data/mod.rs`, the date types in `datetime.rs`, `Currency` and `CurrencyPattern` in `currency.rs`. Changing a type means changing its emitter to match.

## Update README Sections

Values that come from the code (CLDR version, locale count, crate version, feature table) are generated into the READMEs. Code generation refreshes them automatically; after editing `locale-rs/Cargo.toml` by hand, run:

```bash
cargo run -p locale-dev -- readme          # rewrite the generated sections
cargo run -p locale-dev -- readme --check  # only report outdated files (exit code 1)
```

A generated section is an HTML comment `gen:TEMPLATE`, followed by the rendered text and a closing `/gen` comment (view the raw Markdown for examples). Edit the template, never the rendered part: the `readme` CI check fails if they disagree. Placeholders are `{{cldr_version}}`, `{{locale_count}}`, `{{crate_version}}`, `{{crate_version_req}}` and `{{feature_table}}`. A template starting with a line break spans whole lines (badges, code blocks, tables). New Cargo features need a description in `FEATURE_DESCRIPTIONS` in `src/readme.rs`.

The data currently covers <!-- gen:{{locale_count}} -->766<!-- /gen --> locales from CLDR <!-- gen:{{cldr_version}} -->48.2.2<!-- /gen -->.

## Development

### Running Tests

```bash
cargo test -p locale-dev
```

The tests synthesize small CLDR archives in memory, so they need neither network access nor rustfmt.

### Adding Data

To expose more CLDR data in `locale-rs`:

1. Extend the model in `cldr.rs` and read the value from the archive there.
2. Add or extend the handwritten type in `locale-rs` that holds it.
3. Emit it from the matching module in `emit/`, interning repeated values with `Pool`.
4. Regenerate with `cargo run -p locale-dev -- --archive <zip>` and add tests on both sides.

### Debugging

The tool logs through `tracing`; set `RUST_LOG=debug` for more detail.

## Troubleshooting

**Network issues or timeouts**: download the `*-json-full.zip` asset from the [cldr-json releases](https://github.com/unicode-org/cldr-json/releases) yourself and run with `--archive`.

**`rustfmt failed`**: install it with `rustup component add rustfmt`.

**Force a re-download**: delete `cache/cldr-*.zip`.

## Dependencies

- `reqwest` - HTTP client for the GitHub API
- `zip` - Archive reading
- `serde` & `serde_json` - JSON parsing
- `quote`, `proc-macro2`, `syn` & `prettyplease` - Code generation
- `phf_codegen` - The perfect hash map of locale identifiers
- `toml_edit` - Version bookkeeping in `Cargo.toml`
- `thiserror` - Error handling
- `tracing` & `tracing-subscriber` - Logging

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## See Also

- [locale-rs](../locale-rs/README.md) - The library the data is generated for
- [CLDR Project](https://cldr.unicode.org/) - Unicode locale data source
- [CLDR-JSON Repository](https://github.com/unicode-org/cldr-json) - GitHub source
