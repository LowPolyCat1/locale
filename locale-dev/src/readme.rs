//! Keeps the generated parts of the READMEs in sync with the code.
//!
//! A generated region is written as
//!
//! ```text
//! <!-- gen:TEMPLATE -->RENDERED<!-- /gen -->
//! ```
//!
//! `RENDERED` is always `TEMPLATE` with its `{{placeholders}}` filled in. A
//! template that starts with a newline is a block and is rendered verbatim, so
//! it can hold whole lines (badges, code fences, tables). Any other template is
//! inline and trimmed, e.g. `<!-- gen:{{cldr_version}} -->48.2.2<!-- /gen -->`.

use crate::error::{Error, Result};
use crate::version::{read_locale_rs_version, read_workspace_cldr_version};
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;

/// Files that may contain generated regions, relative to the workspace root.
pub const README_FILES: &[&str] = &["README.md", "locale-rs/README.md", "locale-dev/README.md"];

const OPEN: &str = "<!-- gen:";
const OPEN_END: &str = "-->";
const CLOSE: &str = "<!-- /gen -->";

/// Descriptions for the feature table. Every feature of locale-rs needs one,
/// so adding a feature without documenting it fails the README check.
const FEATURE_DESCRIPTIONS: &[(&str, &str)] = &[
    (
        "strum",
        "Derives `strum` traits on `Locale`, e.g. iterating over all locales.",
    ),
    (
        "datetime",
        "Localized date and time formatting (`datetime` module).",
    ),
    (
        "nums",
        "Locale-aware number formatting with native digits (`nums` module).",
    ),
    (
        "currency",
        "Currency formatting from CLDR currency patterns (`currency` module).",
    ),
    ("all", "Every feature above."),
];

/// Values the templates can refer to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub cldr_version: String,
    pub locale_count: usize,
    pub crate_version: String,
    pub feature_table: String,
}

impl Facts {
    fn lookup(&self, key: &str) -> Option<String> {
        match key {
            "cldr_version" => Some(self.cldr_version.clone()),
            "locale_count" => Some(self.locale_count.to_string()),
            "crate_version" => Some(self.crate_version.clone()),
            "crate_version_req" => Some(version_req(&self.crate_version)),
            "feature_table" => Some(self.feature_table.clone()),
            _ => None,
        }
    }
}

/// The Cargo requirement users should write for `version`: `0.3.1` -> `0.3`,
/// `1.2.0` -> `1`. A pre-release such as `0.4.0-rc.1` is only matched by a
/// requirement naming it, so it is returned unchanged.
pub fn version_req(version: &str) -> String {
    if version.contains('-') {
        return version.to_owned();
    }
    let mut parts = version.split('.');
    match (parts.next(), parts.next()) {
        (Some("0"), Some(minor)) => format!("0.{minor}"),
        (Some(major), _) => major.to_owned(),
        _ => version.to_owned(),
    }
}

pub fn collect_facts(workspace_root: &Path) -> Result<Facts> {
    let cldr_version = read_workspace_cldr_version(workspace_root)?.ok_or_else(|| {
        Error::Readme("Missing `[workspace.metadata.cldr] version` in Cargo.toml".into())
    })?;
    let locale_rs = workspace_root.join("locale-rs");
    let locale_count = count_locales(&fs::read_to_string(locale_rs.join("src/data/locales.rs"))?)?;
    let crate_version = read_locale_rs_version(workspace_root)?;
    let manifest: DocumentMut = fs::read_to_string(locale_rs.join("Cargo.toml"))?.parse()?;
    let feature_table = feature_table(&manifest)?;
    Ok(Facts {
        cldr_version,
        locale_count,
        crate_version,
        feature_table,
    })
}

/// Reads `N` from `AVAILABLE_LOCALES: [&str; N]` in the generated `data/locales.rs`.
pub fn count_locales(locale_rs: &str) -> Result<usize> {
    const DECL: &str = "AVAILABLE_LOCALES: [&str;";
    let start = locale_rs
        .find(DECL)
        .ok_or_else(|| Error::Readme("`AVAILABLE_LOCALES` not found in data/locales.rs".into()))?
        + DECL.len();
    let end = start
        + locale_rs[start..]
            .find(']')
            .ok_or_else(|| Error::Readme("Malformed `AVAILABLE_LOCALES` declaration".into()))?;
    locale_rs[start..end]
        .trim()
        .parse()
        .map_err(|e| Error::Readme(format!("Malformed `AVAILABLE_LOCALES` length: {e}")))
}

/// Markdown table of the `[features]` of locale-rs, in manifest order.
pub fn feature_table(manifest: &DocumentMut) -> Result<String> {
    let features = manifest
        .get("features")
        .and_then(|f| f.as_table_like())
        .ok_or_else(|| Error::Readme("Missing `[features]` in locale-rs/Cargo.toml".into()))?;

    let mut rows = vec![
        "| Feature | Enables | Description |".to_owned(),
        "| --- | --- | --- |".to_owned(),
    ];
    let mut seen = Vec::new();
    for (name, item) in features.iter() {
        seen.push(name);
        let description = FEATURE_DESCRIPTIONS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, d)| *d)
            .ok_or_else(|| {
                Error::Readme(format!(
                    "Feature `{name}` has no description; add it to FEATURE_DESCRIPTIONS in locale-dev/src/readme.rs"
                ))
            })?;
        let enables: Vec<String> = item
            .as_array()
            .ok_or_else(|| Error::Readme(format!("Feature `{name}` is not an array")))?
            .iter()
            .filter_map(|v| v.as_str())
            .map(|v| match v.strip_prefix("dep:") {
                Some(dep) => format!("`{dep}` crate"),
                None => format!("`{v}`"),
            })
            .collect();
        let enables = if enables.is_empty() {
            "-".to_owned()
        } else {
            enables.join(", ")
        };
        rows.push(format!("| `{name}` | {enables} | {description} |"));
    }

    if let Some((stale, _)) = FEATURE_DESCRIPTIONS.iter().find(|(n, _)| !seen.contains(n)) {
        return Err(Error::Readme(format!(
            "FEATURE_DESCRIPTIONS lists `{stale}`, which is not a feature of locale-rs"
        )));
    }

    Ok(rows.join("\n"))
}

fn fill(template: &str, facts: &Facts) -> Result<String, String> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let end = rest[start..]
            .find("}}")
            .ok_or_else(|| format!("Unclosed placeholder in template `{template}`"))?
            + start;
        let key = rest[start + 2..end].trim();
        let value = facts
            .lookup(key)
            .ok_or_else(|| format!("Unknown placeholder `{{{{{key}}}}}`"))?;
        out.push_str(&value);
        rest = &rest[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

fn line_of(text: &str, offset: usize) -> usize {
    text[..offset].matches('\n').count() + 1
}

/// Re-renders every generated region of a document.
pub fn render(text: &str, facts: &Facts) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(OPEN) {
        let line = line_of(text, text.len() - rest.len() + start);
        let template_start = start + OPEN.len();
        let template_end = rest[template_start..]
            .find(OPEN_END)
            .ok_or_else(|| format!("line {line}: unterminated `{OPEN}` comment"))?
            + template_start;
        let body_start = template_end + OPEN_END.len();
        let body_end = rest[body_start..]
            .find(CLOSE)
            .ok_or_else(|| format!("line {line}: missing `{CLOSE}`"))?
            + body_start;
        if rest[body_start..body_end].contains(OPEN) {
            return Err(format!("line {line}: nested `{OPEN}` before `{CLOSE}`"));
        }

        let template = &rest[template_start..template_end];
        let template = if template.starts_with('\n') {
            template
        } else {
            template.trim()
        };
        let rendered = fill(template, facts).map_err(|e| format!("line {line}: {e}"))?;

        out.push_str(&rest[..body_start]);
        out.push_str(&rendered);
        out.push_str(CLOSE);
        rest = &rest[body_end + CLOSE.len()..];
    }
    if let Some(pos) = rest.find(CLOSE) {
        let line = line_of(text, text.len() - rest.len() + pos);
        return Err(format!("line {line}: `{CLOSE}` without an opening marker"));
    }
    out.push_str(rest);
    Ok(out)
}

/// Re-renders all README files. With `check`, nothing is written and the
/// returned list names the files that are out of date.
pub fn sync(workspace_root: &Path, check: bool) -> Result<Vec<PathBuf>> {
    let facts = collect_facts(workspace_root)?;
    let mut stale = Vec::new();
    for file in README_FILES {
        let path = workspace_root.join(file);
        let current = fs::read_to_string(&path)?;
        let rendered =
            render(&current, &facts).map_err(|e| Error::Readme(format!("{file}: {e}")))?;
        if rendered == current {
            continue;
        }
        if !check {
            fs::write(&path, &rendered)?;
        }
        stale.push(PathBuf::from(file));
    }
    Ok(stale)
}
