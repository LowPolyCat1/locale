//! Release policy for data updates.
//!
//! `Locale` is an exhaustive enum and formatted output is part of what users
//! rely on, so any change to the generated data (new or removed locales,
//! changed symbols or patterns, even the `CLDR_VERSION` constant) is released
//! as a breaking change. Downstream code then never changes behaviour on a
//! plain `cargo update`, only on an explicit upgrade, where the compiler
//! points at every exhaustive `match` that needs attention.

use crate::error::Result;
use crate::version::Bump;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

/// The files `locale-dev` generates into `locale-rs/src/data`.
pub const GENERATED_FILES: [&str; 4] = ["locales.rs", "numbers.rs", "dates.rs", "currency.rs"];

/// The contents of the generated files at one point in time.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Snapshot {
    files: BTreeMap<&'static str, String>,
}

impl Snapshot {
    /// Reads the generated files in `data_dir`; missing files count as empty.
    pub fn read(data_dir: &Path) -> Result<Self> {
        let mut files = BTreeMap::new();
        for name in GENERATED_FILES {
            let text = match fs::read_to_string(data_dir.join(name)) {
                Ok(text) => text,
                Err(e) if e.kind() == ErrorKind::NotFound => String::new(),
                Err(e) => return Err(e.into()),
            };
            files.insert(name, text);
        }
        Ok(Self { files })
    }

    /// Builds a snapshot from file contents, for tests.
    pub fn from_files(files: &[(&'static str, &str)]) -> Self {
        Self {
            files: files.iter().map(|&(n, t)| (n, t.to_string())).collect(),
        }
    }

    /// The identifiers in `AVAILABLE_LOCALES` of `locales.rs`.
    pub fn locales(&self) -> BTreeSet<String> {
        let text = self.files.get("locales.rs").map_or("", String::as_str);
        let Some(start) = text.find("AVAILABLE_LOCALES") else {
            return BTreeSet::new();
        };
        let body = &text[start..];
        let Some(open) = body.find("= [") else {
            return BTreeSet::new();
        };
        let body = &body[open + 3..];
        let body = &body[..body.find("];").unwrap_or(body.len())];
        // Identifiers never contain quotes, so every odd piece is one.
        body.split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect()
    }
}

/// What a regeneration changed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataChange {
    /// Generated files whose contents differ.
    pub changed_files: Vec<&'static str>,
    /// Locales that are new, i.e. new `Locale` variants.
    pub added_locales: Vec<String>,
    /// Locales that are gone, i.e. removed `Locale` variants.
    pub removed_locales: Vec<String>,
}

impl DataChange {
    pub fn between(old: &Snapshot, new: &Snapshot) -> Self {
        let changed_files = GENERATED_FILES
            .into_iter()
            .filter(|name| old.files.get(name) != new.files.get(name))
            .collect();
        let (old_locales, new_locales) = (old.locales(), new.locales());
        Self {
            changed_files,
            added_locales: new_locales.difference(&old_locales).cloned().collect(),
            removed_locales: old_locales.difference(&new_locales).cloned().collect(),
        }
    }

    /// Any change to the generated data is breaking.
    pub fn bump(&self) -> Bump {
        if self.changed_files.is_empty() {
            Bump::None
        } else {
            Bump::Breaking
        }
    }

    /// A Markdown list of the changes, for logs and pull requests.
    pub fn summary(&self) -> String {
        if self.changed_files.is_empty() {
            return "- The generated data is unchanged.\n".to_string();
        }
        let list = |items: &[String]| {
            items
                .iter()
                .map(|l| format!("`{l}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let mut out = format!(
            "- Changed data files: {}\n",
            self.changed_files
                .iter()
                .map(|f| format!("`data/{f}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        if !self.added_locales.is_empty() {
            out.push_str(&format!(
                "- Added locales ({}): {}\n",
                self.added_locales.len(),
                list(&self.added_locales)
            ));
        }
        if !self.removed_locales.is_empty() {
            out.push_str(&format!(
                "- Removed locales ({}): {}\n",
                self.removed_locales.len(),
                list(&self.removed_locales)
            ));
        }
        if self.added_locales.is_empty() && self.removed_locales.is_empty() {
            out.push_str("- The set of locales is unchanged.\n");
        }
        out
    }
}
