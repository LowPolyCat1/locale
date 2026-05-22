use crate::sanitize_variant;
use crate::version::{
    BumpKind, CldrVersion, bump_locale_rs_version, classify_cldr_bump, parse_version_from_asset,
    read_locale_rs_version, read_workspace_cldr_version, write_workspace_cldr_version,
};
use crate::{
    generate_currency_formatting, generate_datetime_formatting, generate_locales,
    generate_num_formats,
};

use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;
use tempfile::TempDir;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

// ---------------------------------------------------------------------------
// sanitize_variant
// ---------------------------------------------------------------------------

#[test]
fn sanitize_replaces_hyphens_with_underscores() {
    assert_eq!(sanitize_variant("en-GB"), "en_GB");
    assert_eq!(sanitize_variant("zh-Hans-CN"), "zh_Hans_CN");
}

#[test]
fn sanitize_passes_through_simple_names() {
    assert_eq!(sanitize_variant("en"), "en");
    assert_eq!(sanitize_variant("de"), "de");
}

#[test]
fn sanitize_appends_underscore_to_rust_keywords() {
    assert_eq!(sanitize_variant("as"), "as_");
    assert_eq!(sanitize_variant("type"), "type_");
    assert_eq!(sanitize_variant("try"), "try_");
    assert_eq!(sanitize_variant("Self"), "Self_");
    assert_eq!(sanitize_variant("async"), "async_");
}

#[test]
fn sanitize_keyword_check_runs_after_hyphen_replacement() {
    // "do" is a Rust keyword, but "do-XX" rewrites to "do_XX" which is a valid identifier.
    assert_eq!(sanitize_variant("do-XX"), "do_XX");
}

#[test]
fn sanitize_handles_empty_input() {
    assert_eq!(sanitize_variant(""), "");
}

// ---------------------------------------------------------------------------
// version::CldrVersion
// ---------------------------------------------------------------------------

#[test]
fn cldr_version_parses_valid_input() {
    let v = CldrVersion::parse("48.1.0").unwrap();
    assert_eq!(v.major, 48);
    assert_eq!(v.minor, 1);
    assert_eq!(v.patch, 0);
}

#[test]
fn cldr_version_rejects_invalid_input() {
    assert!(CldrVersion::parse("48").is_none());
    assert!(CldrVersion::parse("48.1").is_none());
    assert!(CldrVersion::parse("48.1.0.0").is_none());
    assert!(CldrVersion::parse("48.1.x").is_none());
    assert!(CldrVersion::parse("").is_none());
    assert!(CldrVersion::parse("a.b.c").is_none());
}

#[test]
fn cldr_version_display_is_dotted() {
    let v = CldrVersion {
        major: 1,
        minor: 2,
        patch: 3,
    };
    assert_eq!(format!("{v}"), "1.2.3");
}

#[test]
fn cldr_version_equality() {
    let a = CldrVersion::parse("48.1.0").unwrap();
    let b = CldrVersion::parse("48.1.0").unwrap();
    let c = CldrVersion::parse("48.1.1").unwrap();
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ---------------------------------------------------------------------------
// version::classify_cldr_bump
// ---------------------------------------------------------------------------

fn v(s: &str) -> CldrVersion {
    CldrVersion::parse(s).unwrap()
}

#[test]
fn classify_detects_major() {
    assert_eq!(classify_cldr_bump(v("1.0.0"), v("2.0.0")), BumpKind::Major);
}

#[test]
fn classify_major_takes_precedence() {
    // When major differs, lower segments don't matter for classification.
    assert_eq!(classify_cldr_bump(v("1.5.7"), v("2.0.0")), BumpKind::Major);
    assert_eq!(classify_cldr_bump(v("1.0.0"), v("2.9.9")), BumpKind::Major);
}

#[test]
fn classify_detects_minor() {
    assert_eq!(
        classify_cldr_bump(v("48.0.0"), v("48.1.0")),
        BumpKind::Minor
    );
}

#[test]
fn classify_minor_takes_precedence_over_patch() {
    assert_eq!(
        classify_cldr_bump(v("48.0.0"), v("48.1.5")),
        BumpKind::Minor
    );
}

#[test]
fn classify_detects_patch() {
    assert_eq!(
        classify_cldr_bump(v("48.1.0"), v("48.1.1")),
        BumpKind::Patch
    );
}

#[test]
fn classify_detects_none_when_equal() {
    assert_eq!(classify_cldr_bump(v("48.1.0"), v("48.1.0")), BumpKind::None);
}

#[test]
fn classify_treats_downgrades_as_a_bump() {
    // The function only checks for *difference*, not direction.
    assert_eq!(
        classify_cldr_bump(v("49.0.0"), v("48.0.0")),
        BumpKind::Major
    );
    assert_eq!(
        classify_cldr_bump(v("48.2.0"), v("48.1.0")),
        BumpKind::Minor
    );
}

// ---------------------------------------------------------------------------
// version::parse_version_from_asset
// ---------------------------------------------------------------------------

#[test]
fn parse_asset_extracts_standard_version() {
    assert_eq!(
        parse_version_from_asset("cldr-48.1.0-json-full.zip").as_deref(),
        Some("48.1.0"),
    );
}

#[test]
fn parse_asset_returns_none_for_non_cldr_names() {
    assert_eq!(parse_version_from_asset("foo.zip"), None);
    assert_eq!(parse_version_from_asset("not-a-cldr-asset.zip"), None);
}

#[test]
fn parse_asset_returns_none_when_marker_missing() {
    // No "-json-full" infix means we can't locate the trailing edge of the version.
    assert_eq!(parse_version_from_asset("cldr-48.1.0.zip"), None);
}

#[test]
fn parse_asset_preserves_arbitrary_text_between_markers() {
    // The function just slices between the known prefix/suffix; anything in between is the "version".
    assert_eq!(
        parse_version_from_asset("cldr-48.0.0-rc1-json-full.zip").as_deref(),
        Some("48.0.0-rc1"),
    );
}

// ---------------------------------------------------------------------------
// version: read/write workspace Cargo.toml
// ---------------------------------------------------------------------------

fn make_workspace(workspace_toml: &str, locale_rs_toml: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("Cargo.toml"), workspace_toml).unwrap();
    let locale_dir = dir.path().join("locale-rs");
    fs::create_dir(&locale_dir).unwrap();
    fs::write(locale_dir.join("Cargo.toml"), locale_rs_toml).unwrap();
    dir
}

const MINIMAL_LOCALE_RS: &str = "[package]\nname = \"locale-rs\"\nversion = \"0.1.0\"\n";

#[test]
fn read_workspace_cldr_returns_stored_version() {
    let dir = make_workspace(
        "[workspace]\nmembers = [\"locale-rs\"]\n\n\
         [workspace.metadata.cldr]\nversion = \"48.1.0\"\n",
        MINIMAL_LOCALE_RS,
    );
    let version = read_workspace_cldr_version(dir.path()).unwrap();
    assert_eq!(version.as_deref(), Some("48.1.0"));
}

#[test]
fn read_workspace_cldr_returns_none_when_missing() {
    let dir = make_workspace(
        "[workspace]\nmembers = [\"locale-rs\"]\n",
        MINIMAL_LOCALE_RS,
    );
    assert!(read_workspace_cldr_version(dir.path()).unwrap().is_none());
}

#[test]
fn write_workspace_cldr_creates_table_when_absent() {
    let dir = make_workspace(
        "[workspace]\nmembers = [\"locale-rs\"]\n",
        MINIMAL_LOCALE_RS,
    );
    write_workspace_cldr_version(dir.path(), "49.0.0").unwrap();
    let version = read_workspace_cldr_version(dir.path()).unwrap();
    assert_eq!(version.as_deref(), Some("49.0.0"));
}

#[test]
fn write_workspace_cldr_overwrites_existing_value() {
    let dir = make_workspace(
        "[workspace]\nmembers = [\"locale-rs\"]\n\n\
         [workspace.metadata.cldr]\nversion = \"48.0.0\"\n",
        MINIMAL_LOCALE_RS,
    );
    write_workspace_cldr_version(dir.path(), "49.0.0").unwrap();
    let version = read_workspace_cldr_version(dir.path()).unwrap();
    assert_eq!(version.as_deref(), Some("49.0.0"));
}

#[test]
fn write_workspace_cldr_preserves_unrelated_keys() {
    let dir = make_workspace(
        "[workspace]\nmembers = [\"locale-rs\"]\nresolver = \"3\"\n",
        MINIMAL_LOCALE_RS,
    );
    write_workspace_cldr_version(dir.path(), "48.2.0").unwrap();
    let content = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
    assert!(content.contains("resolver = \"3\""));
    assert!(content.contains("48.2.0"));
}

// ---------------------------------------------------------------------------
// version: bump_locale_rs_version
// ---------------------------------------------------------------------------

fn workspace_with_locale_version(starting: &str) -> TempDir {
    make_workspace(
        "[workspace]\nmembers = [\"locale-rs\"]\n",
        &format!("[package]\nname = \"locale-rs\"\nversion = \"{starting}\"\n"),
    )
}

#[test]
fn read_locale_rs_returns_current_version() {
    let dir = workspace_with_locale_version("1.2.3");
    assert_eq!(read_locale_rs_version(dir.path()).unwrap(), "1.2.3");
}

#[test]
fn bump_none_leaves_version_unchanged_and_does_not_rewrite() {
    let dir = workspace_with_locale_version("1.2.3");
    let path = dir.path().join("locale-rs/Cargo.toml");
    let before = fs::read_to_string(&path).unwrap();

    let (old, new) = bump_locale_rs_version(dir.path(), BumpKind::None).unwrap();
    assert_eq!(old, "1.2.3");
    assert_eq!(new, "1.2.3");

    // On BumpKind::None the file should not be touched.
    let after = fs::read_to_string(&path).unwrap();
    assert_eq!(before, after);
}

#[test]
fn bump_major_on_stable_resets_lower_components() {
    let dir = workspace_with_locale_version("1.2.3");
    let (old, new) = bump_locale_rs_version(dir.path(), BumpKind::Major).unwrap();
    assert_eq!(old, "1.2.3");
    assert_eq!(new, "2.0.0");
    assert_eq!(read_locale_rs_version(dir.path()).unwrap(), "2.0.0");
}

#[test]
fn bump_minor_on_stable_resets_patch() {
    let dir = workspace_with_locale_version("1.2.3");
    let (_, new) = bump_locale_rs_version(dir.path(), BumpKind::Minor).unwrap();
    assert_eq!(new, "1.3.0");
}

#[test]
fn bump_patch_on_stable_increments_patch() {
    let dir = workspace_with_locale_version("1.2.3");
    let (_, new) = bump_locale_rs_version(dir.path(), BumpKind::Patch).unwrap();
    assert_eq!(new, "1.2.4");
}

#[test]
fn bump_major_on_zerover_promotes_minor() {
    // 0.x.y semver convention: "breaking" still keeps major at 0 and bumps minor.
    let dir = workspace_with_locale_version("0.2.3");
    let (_, new) = bump_locale_rs_version(dir.path(), BumpKind::Major).unwrap();
    assert_eq!(new, "0.3.0");
}

#[test]
fn bump_minor_on_zerover_also_promotes_minor() {
    let dir = workspace_with_locale_version("0.2.3");
    let (_, new) = bump_locale_rs_version(dir.path(), BumpKind::Minor).unwrap();
    assert_eq!(new, "0.3.0");
}

#[test]
fn bump_patch_on_zerover_increments_patch() {
    let dir = workspace_with_locale_version("0.2.3");
    let (_, new) = bump_locale_rs_version(dir.path(), BumpKind::Patch).unwrap();
    assert_eq!(new, "0.2.4");
}

#[test]
fn bump_fails_when_version_is_not_three_parts() {
    let dir = workspace_with_locale_version("1.2");
    assert!(bump_locale_rs_version(dir.path(), BumpKind::Patch).is_err());
}

#[test]
fn bump_fails_when_version_has_non_numeric_component() {
    let dir = workspace_with_locale_version("1.2.beta");
    assert!(bump_locale_rs_version(dir.path(), BumpKind::Patch).is_err());
}

// ---------------------------------------------------------------------------
// Integration: generate_* against a synthesised minimal CLDR zip
// ---------------------------------------------------------------------------

/// Build an in-memory zip whose layout looks like a CLDR `*-json-full.zip`:
/// `cldr-misc-full/main/{locale}/` directory entries. The generators only
/// scrape locale names from these directory paths, so this is enough to
/// exercise their core flow without bundling real CLDR JSON.
fn make_minimal_cldr_zip(locales: &[&str]) -> Vec<u8> {
    let mut buf = Vec::<u8>::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default();
        for loc in locales {
            writer
                .add_directory(format!("cldr-misc-full/main/{loc}/"), opts)
                .unwrap();
        }
        writer.finish().unwrap();
    }
    buf
}

fn run_locales(zip: Vec<u8>, asset: &str, out: &Path) {
    generate_locales::run(zip, asset, out.to_str().unwrap()).unwrap();
}

#[test]
fn generate_locales_emits_enum_with_fallbacks_and_region_codes() {
    let zip = make_minimal_cldr_zip(&["en", "en-GB", "de", "zh-Hans"]);
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("locale.rs");
    run_locales(zip, "cldr-48.0.0-json-full.zip", &out);

    let contents = fs::read_to_string(&out).unwrap();

    // Enum and variants
    assert!(contents.contains("pub enum Locale"));
    assert!(contents.contains("    en,\n"));
    assert!(contents.contains("    en_GB,\n"));
    assert!(contents.contains("    de,\n"));
    assert!(contents.contains("    zh_Hans,\n"));

    // Fallback chain: child locale → parent
    assert!(contents.contains("Locale::en_GB => Some(Locale::en)"));
    // `zh` is not in the input, so `zh-Hans` falls back to None.
    assert!(!contents.contains("Locale::zh_Hans => Some(Locale::zh)"));
    assert!(contents.contains("Locale::zh_Hans => None"));
    // Top-level locale also has no fallback.
    assert!(contents.contains("Locale::en => None"));

    // Region code extraction
    assert!(contents.contains("Locale::en_GB => Some(\"GB\")"));
    assert!(contents.contains("Locale::en => None"));

    // Language code
    assert!(contents.contains("Locale::en_GB => \"en\""));
    assert!(contents.contains("Locale::zh_Hans => \"zh\""));

    // Source asset constant is wired through
    assert!(contents.contains("SOURCE_ASSET: &str = \"cldr-48.0.0-json-full.zip\""));

    // AVAILABLE_LOCALES length matches input
    assert!(contents.contains("AVAILABLE_LOCALES: [&str; 4]"));
}

#[test]
fn generate_locales_sanitises_keyword_locales() {
    // "as" (Assamese) is a Rust keyword and must get a trailing underscore.
    let zip = make_minimal_cldr_zip(&["as"]);
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("locale.rs");
    run_locales(zip, "cldr-x-json-full.zip", &out);

    let contents = fs::read_to_string(&out).unwrap();
    assert!(contents.contains("    as_,\n"));
    assert!(contents.contains("Locale::as_ => \"as\""));
    assert!(contents.contains("\"as\" => Ok(Locale::as_)"));
}

#[test]
fn generate_num_formats_falls_back_to_defaults_when_data_missing() {
    // No numbers.json shipped → every locale should land on the default
    // separators / minus / grouping sizes.
    let zip = make_minimal_cldr_zip(&["en", "de"]);
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("num_formats.rs");
    generate_num_formats::run(zip, "ignored", out.to_str().unwrap()).unwrap();

    let contents = fs::read_to_string(&out).unwrap();
    assert!(contents.contains("pub fn decimal_separator"));
    assert!(contents.contains("pub fn grouping_separator"));
    assert!(contents.contains("pub fn minus_sign"));
    assert!(contents.contains("Locale::en => \".\""));
    assert!(contents.contains("Locale::de => \".\""));
    assert!(contents.contains("Locale::en => \"-\""));
    // Default grouping size is [3]
    assert!(contents.contains("Locale::en => &[3]"));
    // No numeric system override means digits() returns None.
    assert!(contents.contains("Locale::en => None"));
}

#[test]
fn generate_datetime_formatting_falls_back_to_defaults_when_data_missing() {
    let zip = make_minimal_cldr_zip(&["en"]);
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("datetime_formats.rs");
    generate_datetime_formatting::run(zip, "ignored", out.to_str().unwrap()).unwrap();

    let contents = fs::read_to_string(&out).unwrap();
    // Default fallback strings are hard-coded in the generator.
    assert!(contents.contains("Locale::en => \"y-MM-dd\""));
    assert!(contents.contains("Locale::en => \"HH:mm:ss\""));
    assert!(contents.contains("Locale::en => (\"AM\", \"PM\")"));
    assert!(contents.contains("pub fn format_date"));
    assert!(contents.contains("pub fn format_time"));
}

#[test]
fn generate_currency_formatting_uses_defaults_when_data_missing() {
    let zip = make_minimal_cldr_zip(&["en"]);
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("currency_formats.rs");
    generate_currency_formatting::run(zip, "ignored", out.to_str().unwrap()).unwrap();

    let contents = fs::read_to_string(&out).unwrap();
    // No likelySubtags / currencyData / currencies.json → falls back to USD / pattern default.
    assert!(contents.contains("pub fn format_currency"));
    assert!(contents.contains("pub fn currency_standard_pattern"));
    assert!(contents.contains("pub fn default_currency_symbol"));
    // Default symbol when no likelySubtags/currencies.json data is present.
    assert!(contents.contains("Locale::en => \"USD\""));
    // Default pattern is emitted via Debug formatting; ¤ is a printable Unicode
    // character so it survives as-is. Just check both ends of the pattern.
    assert!(contents.contains("\u{00a4}#,##0.00"));
}

#[test]
fn generate_num_formats_with_real_numbering_systems_emits_digit_table() {
    // Build a zip that *does* include numberingSystems.json plus a locale that
    // points at a non-latn system, so the digits() arm should be populated.
    let mut buf = Vec::<u8>::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default();

        writer
            .add_directory("cldr-misc-full/main/ar/", opts)
            .unwrap();

        writer
            .start_file("cldr-core/supplemental/numberingSystems.json", opts)
            .unwrap();
        // Regular raw string (not byte) — Arabic digits are non-ASCII so a
        // `br"…"` literal would be rejected.
        writer
            .write_all(
                r#"{
                  "supplemental": {
                    "numberingSystems": {
                      "arab": { "_type": "numeric", "_digits": "٠١٢٣٤٥٦٧٨٩" }
                    }
                  }
                }"#
                .as_bytes(),
            )
            .unwrap();

        writer
            .start_file("cldr-numbers-full/main/ar/numbers.json", opts)
            .unwrap();
        // Use `br###"..."###` so the embedded `"#` and `"##` sequences in the
        // pattern string don't terminate the raw byte string.
        writer
            .write_all(
                r###"{
                  "main": {
                    "ar": {
                      "numbers": {
                        "defaultNumberingSystem": "arab",
                        "symbols-numberSystem-arab": {
                          "decimal": ",",
                          "group": ".",
                          "minusSign": "-"
                        },
                        "decimalFormats-numberSystem-arab": {
                          "standard": "#,##,##0.###"
                        }
                      }
                    }
                  }
                }"###
                    .as_bytes(),
            )
            .unwrap();

        writer.finish().unwrap();
    }

    let dir = TempDir::new().unwrap();
    let out = dir.path().join("num_formats.rs");
    generate_num_formats::run(buf, "ignored", out.to_str().unwrap()).unwrap();

    let contents = fs::read_to_string(&out).unwrap();
    // Symbols from the JSON
    assert!(contents.contains("Locale::ar => \",\""));
    assert!(contents.contains("Locale::ar => \".\""));
    // Indian-style grouping pattern → [3, 2]
    assert!(contents.contains("Locale::ar => &[3, 2]"));
    // Digit table is emitted as a `Some([...])` array. We just check the marker.
    assert!(contents.contains("Locale::ar => Some(["));
}
