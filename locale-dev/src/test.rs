use crate::cldr::Cldr;
use crate::policy::{DataChange, Snapshot};
use crate::sanitize_variant;
use crate::version::{
    Bump, bump_locale_rs_version, parse_version_from_asset, read_locale_rs_version,
    read_workspace_cldr_version, write_workspace_cldr_version,
};
use crate::{emit, readme};

use std::fs;
use std::io::{Cursor, Write};
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

    let (old, new) = bump_locale_rs_version(dir.path(), Bump::None).unwrap();
    assert_eq!((old.as_str(), new.as_str()), ("1.2.3", "1.2.3"));
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[test]
fn breaking_bump_on_stable_bumps_major() {
    let dir = workspace_with_locale_version("1.2.3");
    let (old, new) = bump_locale_rs_version(dir.path(), Bump::Breaking).unwrap();
    assert_eq!((old.as_str(), new.as_str()), ("1.2.3", "2.0.0"));
    assert_eq!(read_locale_rs_version(dir.path()).unwrap(), "2.0.0");
}

#[test]
fn breaking_bump_on_zerover_bumps_minor() {
    // Cargo treats 0.x -> 0.(x+1) as the breaking step.
    let dir = workspace_with_locale_version("0.2.3");
    let (_, new) = bump_locale_rs_version(dir.path(), Bump::Breaking).unwrap();
    assert_eq!(new, "0.3.0");
}

#[test]
fn breaking_bump_on_pre_release_counts_up() {
    let dir = workspace_with_locale_version("0.4.0-rc.1");
    let (_, new) = bump_locale_rs_version(dir.path(), Bump::Breaking).unwrap();
    assert_eq!(new, "0.4.0-rc.2");
}

#[test]
fn bump_fails_on_malformed_versions() {
    for version in ["1.2", "1.2.beta", "1.2.3-rc"] {
        let dir = workspace_with_locale_version(version);
        assert!(
            bump_locale_rs_version(dir.path(), Bump::Breaking).is_err(),
            "{version}"
        );
    }
}

// ---------------------------------------------------------------------------
// policy: every data change is breaking
// ---------------------------------------------------------------------------

fn locales_rs(ids: &[&str]) -> String {
    let list: Vec<String> = ids.iter().map(|id| format!("    \"{id}\",")).collect();
    format!(
        "pub const CLDR_VERSION: &str = \"48.0.0\";\npub const AVAILABLE_LOCALES: [&str; {}] = [\n{}\n];\n",
        ids.len(),
        list.join("\n")
    )
}

#[test]
fn policy_reads_locales_from_generated_file() {
    let snapshot = Snapshot::from_files(&[("locales.rs", &locales_rs(&["de", "en", "en-GB"]))]);
    let locales: Vec<String> = snapshot.locales().into_iter().collect();
    assert_eq!(locales, ["de", "en", "en-GB"]);
    assert!(Snapshot::default().locales().is_empty());
}

#[test]
fn policy_unchanged_data_needs_no_release() {
    let files = locales_rs(&["de", "en"]);
    let snapshot = Snapshot::from_files(&[("locales.rs", &files), ("numbers.rs", "x")]);
    let change = DataChange::between(&snapshot, &snapshot.clone());
    assert_eq!(change, DataChange::default());
    assert_eq!(change.bump(), Bump::None);
    assert!(change.summary().contains("unchanged"));
}

#[test]
fn policy_added_and_removed_locales_are_breaking() {
    let old = Snapshot::from_files(&[("locales.rs", &locales_rs(&["de", "en", "xx"]))]);
    let new = Snapshot::from_files(&[("locales.rs", &locales_rs(&["de", "en", "yy", "zz"]))]);
    let change = DataChange::between(&old, &new);
    assert_eq!(change.bump(), Bump::Breaking);
    assert_eq!(change.changed_files, ["locales.rs"]);
    assert_eq!(change.added_locales, ["yy", "zz"]);
    assert_eq!(change.removed_locales, ["xx"]);
    let summary = change.summary();
    assert!(
        summary.contains("Added locales (2): `yy`, `zz`"),
        "{summary}"
    );
    assert!(summary.contains("Removed locales (1): `xx`"), "{summary}");
}

#[test]
fn policy_output_only_changes_are_breaking_too() {
    // Same locales, different symbols: no compile error downstream, but the
    // output changes, so it still needs a breaking release.
    let locales = locales_rs(&["de", "en"]);
    let old = Snapshot::from_files(&[("locales.rs", &locales), ("numbers.rs", "group: \".\"")]);
    let new = Snapshot::from_files(&[("locales.rs", &locales), ("numbers.rs", "group: \" \"")]);
    let change = DataChange::between(&old, &new);
    assert_eq!(change.bump(), Bump::Breaking);
    assert_eq!(change.changed_files, ["numbers.rs"]);
    assert!(change.added_locales.is_empty() && change.removed_locales.is_empty());
    assert!(change.summary().contains("set of locales is unchanged"));
}

#[test]
fn policy_reads_the_real_data_directory() {
    let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../locale-rs/src/data");
    let snapshot = Snapshot::read(&data).unwrap();
    let declared = readme::count_locales(&fs::read_to_string(data.join("locales.rs")).unwrap());
    assert_eq!(snapshot.locales().len(), declared.unwrap());
    assert!(snapshot.locales().contains("en-GB"));
}

// ---------------------------------------------------------------------------
// Integration: generate_* against a synthesised minimal CLDR zip
// ---------------------------------------------------------------------------

/// Supplemental files every archive needs; the per-locale files are optional.
const SUPPLEMENTAL: &[(&str, &str)] = &[
    (
        "cldr-core/supplemental/numberingSystems.json",
        r#"{"supplemental":{"numberingSystems":{
            "arab":{"_type":"numeric","_digits":"٠١٢٣٤٥٦٧٨٩"},
            "latn":{"_type":"numeric","_digits":"0123456789"}}}}"#,
    ),
    (
        "cldr-core/supplemental/likelySubtags.json",
        r#"{"supplemental":{"likelySubtags":{
            "en":"en-Latn-US","de":"de-Latn-DE","ar":"ar-Arab-EG","zh":"zh-Hans-CN"}}}"#,
    ),
    (
        "cldr-core/supplemental/parentLocales.json",
        r#"{"supplemental":{"parentLocales":{"parentLocale":{"en-IN":"en-001"}}}}"#,
    ),
    (
        "cldr-core/supplemental/currencyData.json",
        r#"{"supplemental":{"currencyData":{
            "fractions":{"DEFAULT":{"_digits":"2"},"JPY":{"_digits":"0"}},
            "region":{
                "US":[{"USD":{"_from":"1792-01-01"}}],
                "DE":[{"DEM":{"_to":"2002-02-28"}},{"EUR":{"_from":"1999-01-01"}}],
                "AT":[{"EUR":{"_from":"1999-01-01"}}],
                "EG":[{"EGP":{"_from":"1885-11-14"}}]}}}}"#,
    ),
];

/// Builds an in-memory zip with the layout of a CLDR `*-json-full.zip`:
/// the supplemental files, a `cldr-misc-full/main/{locale}/` entry per
/// locale, and any extra `(path, contents)` files.
fn make_cldr_zip(locales: &[&str], files: &[(&str, &str)]) -> Vec<u8> {
    let mut buf = Vec::<u8>::new();
    {
        let mut writer = ZipWriter::new(Cursor::new(&mut buf));
        let opts = SimpleFileOptions::default();
        for loc in locales {
            writer
                .start_file(format!("cldr-misc-full/main/{loc}/characters.json"), opts)
                .unwrap();
            writer.write_all(b"{}").unwrap();
        }
        for (path, contents) in SUPPLEMENTAL.iter().chain(files) {
            writer.start_file(*path, opts).unwrap();
            writer.write_all(contents.as_bytes()).unwrap();
        }
        writer.finish().unwrap();
    }
    buf
}

/// Writes an emitted file, checks that it is valid Rust and returns it with
/// all spaces removed, since rustfmt has not normalized them yet.
fn render(file: emit::RustFile) -> String {
    let dir = TempDir::new().unwrap();
    let path = file.write(&dir.path().join("out.rs")).unwrap();
    let text = fs::read_to_string(path).unwrap();
    syn::parse_file(&text).unwrap_or_else(|e| panic!("invalid Rust ({e}):\n{text}"));
    text.replace(' ', "")
}

#[test]
fn cldr_model_discovers_locales_and_parents() {
    let zip = make_cldr_zip(&["en", "en-001", "en-GB", "en-IN", "de", "zh-Hans"], &[]);
    let cldr = Cldr::from_zip(zip).unwrap();

    let names: Vec<&str> = cldr.locales.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(names, ["de", "en", "en-001", "en-GB", "en-IN", "zh-Hans"]);

    let parent = |name: &str| {
        let l = cldr.locales.iter().find(|l| l.name == name).unwrap();
        l.parent.map(|p| cldr.locales[p].name.as_str())
    };
    assert_eq!(parent("en-GB"), Some("en"));
    assert_eq!(parent("en-IN"), Some("en-001"));
    assert_eq!(parent("en-001"), Some("en"));
    // `zh` is not in the archive, and zh-Hans has no other ancestor.
    assert_eq!(parent("zh-Hans"), None);
    assert_eq!(parent("en"), None);
}

#[test]
fn cldr_model_uses_defaults_when_data_missing() {
    let cldr = Cldr::from_zip(make_cldr_zip(&["en", "de-AT"], &[])).unwrap();
    let en = &cldr.locales[1];
    assert_eq!(en.numbers, crate::cldr::NumberData::default());
    assert_eq!(en.dates, crate::cldr::DateData::default());
    assert_eq!(en.currency_pattern, "¤#,##0.00");
    assert_eq!(en.default_currency, "USD");
    // de-AT has its own region, whose tender is EUR.
    assert_eq!(cldr.locales[0].default_currency, "EUR");
    assert_eq!(cldr.fraction_digits.get("JPY"), Some(&0));
    assert!(!cldr.fraction_digits.contains_key("DEFAULT"));
}

#[test]
fn cldr_model_rejects_empty_archives() {
    assert!(Cldr::from_zip(make_cldr_zip(&[], &[])).is_err());
    assert!(Cldr::from_zip(b"not a zip".to_vec()).is_err());
}

#[test]
fn emit_locales_writes_enum_map_and_parents() {
    let zip = make_cldr_zip(&["en", "en-GB", "as"], &[]);
    let cldr = Cldr::from_zip(zip).unwrap();
    let text = render(emit::locales::emit(&cldr, "48.0.0").unwrap());

    assert!(text.contains("pubconstCLDR_VERSION:&str=\"48.0.0\";"));
    assert!(text.contains("AVAILABLE_LOCALES:[&str;3]"));
    assert!(text.contains("pubenumLocale"));
    // "as" (Assamese) is a Rust keyword and must get a trailing underscore.
    assert!(text.contains("as_,"));
    assert!(text.contains("en_GB,"));
    assert!(text.contains("Locale::as_"));
    // One parent row per locale, commented with the locale.
    assert!(text.contains("None,//en\n"));
    assert!(text.contains("Some(Locale::en),//en-GB\n"));
}

#[test]
fn emit_numbers_reads_symbols_digits_and_grouping() {
    let numbers = r###"{"main":{"ar":{"numbers":{
        "defaultNumberingSystem":"arab",
        "minimumGroupingDigits":"2",
        "symbols-numberSystem-arab":{"decimal":"٫","group":"٬","minusSign":"؜-"},
        "decimalFormats-numberSystem-arab":{"standard":"#,##,##0.###"},
        "currencyFormats-numberSystem-arab":{"standard":"#,##0.00 ¤;-#,##0.00 ¤"}}}}}"###;
    let zip = make_cldr_zip(
        &["ar", "en"],
        &[("cldr-numbers-full/main/ar/numbers.json", numbers)],
    );
    let cldr = Cldr::from_zip(zip).unwrap();

    let ar = &cldr.locales[0];
    assert_eq!(ar.numbers.decimal, "٫");
    assert_eq!(ar.numbers.digits.map(|d| d[1]), Some('١'));
    assert_eq!(ar.currency_pattern, "#,##0.00 ¤;-#,##0.00 ¤");
    assert_eq!(ar.numbers.min_grouping_digits, 2);
    // Locales without the key keep the CLDR default of 1.
    assert_eq!(cldr.locales[1].numbers.min_grouping_digits, 1);

    let text = render(emit::numbers::emit(&cldr, "48.0.0").unwrap());
    assert!(text.contains("primary:3"));
    assert!(text.contains("secondary:2"));
    assert!(text.contains("min_grouping_digits:2"));
    assert!(text.contains("min_grouping_digits:1"));
    assert!(text.contains("'٠'"));
    // Identical symbol sets are emitted once: en shares nothing with ar.
    assert_eq!(text.matches("=NumberSymbols{").count(), 2);
    assert!(text.contains("NUMBER_SYMBOLS:[&NumberSymbols;2]"));
}

#[test]
fn emit_dates_pre_parses_patterns() {
    let gregorian = r#"{"main":{"en":{"dates":{"calendars":{"gregorian":{
        "months":{"format":{"wide":{"1":"January"},"abbreviated":{"1":"Jan"}}},
        "days":{"format":{"wide":{"sun":"Sunday"}}},
        "dayPeriods":{"format":{"wide":{"am":"AM","pm":"PM"}}},
        "dateFormats":{"medium":"MMM d, y"},
        "timeFormats":{"medium":"h:mm:ss a"}}}}}}}"#;
    let zip = make_cldr_zip(
        &["en", "de"],
        &[("cldr-dates-full/main/en/ca-gregorian.json", gregorian)],
    );
    let cldr = Cldr::from_zip(zip).unwrap();
    // de has no calendar data and gets the defaults.
    assert_eq!(cldr.locales[0].dates.date_pattern, "y-MM-dd");

    let text = render(emit::dates::emit(&cldr, "48.0.0").unwrap());
    assert!(text.contains("DatePart::Month(3)"));
    assert!(text.contains("DatePart::Literal(\",\")"));
    assert!(text.contains("DatePart::Hour12(1)"));
    assert!(text.contains("DatePart::DayPeriod"));
    assert!(text.contains("source:\"MMMd,y\""));
}

#[test]
fn emit_currency_stores_only_symbol_overrides() {
    let currencies = |loc: &str, eur: &str| {
        format!(
            r#"{{"main":{{"{loc}":{{"numbers":{{"currencies":{{
                "EUR":{{"symbol":"{eur}"}},"USD":{{"symbol":"US$"}},"JPY":{{}}}}}}}}}}}}"#
        )
    };
    let de = currencies("de", "€");
    let de_at = currencies("de-AT", "€");
    let en = currencies("en", "€");
    let zip = make_cldr_zip(
        &["de", "de-AT", "en"],
        &[
            ("cldr-numbers-full/main/de/currencies.json", &de),
            ("cldr-numbers-full/main/de-AT/currencies.json", &de_at),
            ("cldr-numbers-full/main/en/currencies.json", &en),
        ],
    );
    let cldr = Cldr::from_zip(zip).unwrap();

    let overrides = emit::currency::symbol_overrides(&cldr);
    // de-AT inherits everything from de; JPY without a symbol is its code.
    assert!(overrides[1].is_empty());
    assert_eq!(
        overrides[0],
        [
            ("EUR".to_string(), "€".to_string()),
            ("USD".to_string(), "US$".to_string())
        ]
    );
    assert_eq!(
        emit::currency::resolve_symbol(&cldr, &overrides, 1, "USD"),
        "US$"
    );
    assert_eq!(
        emit::currency::resolve_symbol(&cldr, &overrides, 1, "JPY"),
        "JPY"
    );

    let text = render(emit::currency::emit(&cldr, "48.0.0").unwrap());
    assert!(text.contains("Currency(*b\"EUR\")"));
    assert!(text.contains("FRACTION_DIGITS:[(Currency,u8);1]"));
    assert!(text.contains("&[],//de-AT"));
}

// ---------------------------------------------------------------------------
// readme
// ---------------------------------------------------------------------------

fn readme_facts() -> readme::Facts {
    readme::Facts {
        cldr_version: "48.2.2".into(),
        locale_count: 766,
        crate_version: "0.3.1".into(),
        feature_table: "| a |\n| --- |".into(),
    }
}

#[test]
fn readme_version_req_matches_cargo_compatibility() {
    assert_eq!(readme::version_req("0.3.1"), "0.3");
    assert_eq!(readme::version_req("1.2.0"), "1");
    // A pre-release is only matched by a requirement naming it.
    assert_eq!(readme::version_req("0.4.0-rc.1"), "0.4.0-rc.1");
}

#[test]
fn readme_renders_inline_regions() {
    let text = "CLDR <!-- gen:{{cldr_version}} -->48.1.0<!-- /gen --> with <!-- gen: {{ locale_count }} --><!-- /gen --> locales";
    assert_eq!(
        readme::render(text, &readme_facts()).unwrap(),
        "CLDR <!-- gen:{{cldr_version}} -->48.2.2<!-- /gen --> with <!-- gen: {{ locale_count }} -->766<!-- /gen --> locales",
    );
}

#[test]
fn readme_renders_block_regions_verbatim() {
    let text = "<!-- gen:\nlocale-rs = \"{{crate_version_req}}\"\n{{feature_table}}\n-->\nstale\n<!-- /gen -->\n";
    assert_eq!(
        readme::render(text, &readme_facts()).unwrap(),
        "<!-- gen:\nlocale-rs = \"{{crate_version_req}}\"\n{{feature_table}}\n-->\nlocale-rs = \"0.3\"\n| a |\n| --- |\n<!-- /gen -->\n",
    );
}

#[test]
fn readme_render_is_idempotent() {
    let text = "a <!-- gen:{{cldr_version}} -->x<!-- /gen --> b";
    let once = readme::render(text, &readme_facts()).unwrap();
    assert_eq!(readme::render(&once, &readme_facts()).unwrap(), once);
}

#[test]
fn readme_render_rejects_malformed_markers() {
    let facts = readme_facts();
    let err = readme::render("x\n<!-- gen:{{nope}} --><!-- /gen -->", &facts).unwrap_err();
    assert!(err.contains("line 2") && err.contains("nope"), "{err}");
    assert!(readme::render("<!-- gen:{{cldr_version}} -->", &facts).is_err());
    assert!(readme::render("<!-- gen:{{cldr_version} --><!-- /gen -->", &facts).is_err());
    assert!(readme::render("<!-- /gen -->", &facts).is_err());
    assert!(
        readme::render(
            "<!-- gen:a -->x <!-- gen:b -->y<!-- /gen --><!-- /gen -->",
            &facts
        )
        .is_err()
    );
}

#[test]
fn readme_counts_available_locales() {
    let src = "pub const AVAILABLE_LOCALES: [&str; 3] = [\"a\", \"b\", \"c\"];";
    assert_eq!(readme::count_locales(src).unwrap(), 3);
    assert!(readme::count_locales("pub enum Locale {}").is_err());
}

#[test]
fn readme_feature_table_lists_features_in_manifest_order() {
    let manifest: toml_edit::DocumentMut = r#"
[features]
nums = []
strum = ["dep:strum", "dep:strum_macros"]
currency = ["nums"]
datetime = []
all = ["datetime", "nums"]
"#
    .parse()
    .unwrap();
    let table = readme::feature_table(&manifest).unwrap();
    let lines: Vec<&str> = table.lines().collect();
    assert_eq!(lines[0], "| Feature | Enables | Description |");
    assert!(lines[2].starts_with("| `nums` | - |"));
    assert!(lines[3].starts_with("| `strum` | `strum` crate, `strum_macros` crate |"));
    assert!(lines[4].starts_with("| `currency` | `nums` |"));
    assert!(lines[6].starts_with("| `all` | `datetime`, `nums` |"));
}

#[test]
fn readme_feature_table_requires_descriptions() {
    let undocumented: toml_edit::DocumentMut =
        "[features]\nnums = []\nshiny = []\n".parse().unwrap();
    let err = readme::feature_table(&undocumented)
        .unwrap_err()
        .to_string();
    assert!(err.contains("shiny"), "{err}");

    // A description for a feature that no longer exists is stale.
    let missing: toml_edit::DocumentMut = "[features]\nnums = []\n".parse().unwrap();
    assert!(readme::feature_table(&missing).is_err());
}

#[test]
fn readme_sync_checks_and_writes_workspace() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\n[workspace.metadata.cldr]\nversion = \"48.2.2\"\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("locale-rs/src/data")).unwrap();
    fs::create_dir_all(root.join("locale-dev")).unwrap();
    fs::write(
        root.join("locale-rs/Cargo.toml"),
        "[package]\nversion = \"0.3.1\"\n[features]\nstrum = []\ndatetime = []\nnums = []\ncurrency = []\nall = []\n",
    )
    .unwrap();
    fs::write(
        root.join("locale-rs/src/data/locales.rs"),
        "pub const AVAILABLE_LOCALES: [&str; 2] = [\"en\", \"de\"];",
    )
    .unwrap();
    let stale = "CLDR <!-- gen:{{cldr_version}} -->48.1.0<!-- /gen -->\n";
    for file in readme::README_FILES {
        fs::write(root.join(file), stale).unwrap();
    }

    let reported = readme::sync(root, true).unwrap();
    assert_eq!(reported.len(), readme::README_FILES.len());
    assert_eq!(fs::read_to_string(root.join("README.md")).unwrap(), stale);

    readme::sync(root, false).unwrap();
    assert_eq!(
        fs::read_to_string(root.join("README.md")).unwrap(),
        "CLDR <!-- gen:{{cldr_version}} -->48.2.2<!-- /gen -->\n",
    );
    assert!(readme::sync(root, true).unwrap().is_empty());
}
