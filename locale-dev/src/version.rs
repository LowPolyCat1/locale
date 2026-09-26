use crate::error::{Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Table, value};

/// How to change the `locale-rs` version.
///
/// There is no minor or patch bump for data updates: every change to the
/// generated data is released as a breaking change, so no downstream build
/// or output changes without an explicit upgrade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bump {
    /// The next breaking release: `0.4.2` -> `0.5.0`, `1.2.3` -> `2.0.0`.
    /// A pre-release counts up instead: `0.5.0-rc.1` -> `0.5.0-rc.2`,
    /// since `0.5.0` itself is not out yet.
    Breaking,
    /// Leave the version as it is.
    None,
}

/// The version after applying `bump` to `current`.
pub fn next_version(current: &str, bump: Bump) -> Result<String> {
    let invalid = |why: &str| Error::Version(format!("Cannot bump `{current}`: {why}"));
    let (core, pre) = match current.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (current, None),
    };
    let parts: Vec<u64> = core
        .split('.')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|_| invalid("expected MAJOR.MINOR.PATCH"))?;
    let &[major, minor, _] = parts.as_slice() else {
        return Err(invalid("expected MAJOR.MINOR.PATCH"));
    };

    Ok(match (bump, pre) {
        (Bump::None, _) => current.to_string(),
        (Bump::Breaking, Some(pre)) => {
            let (label, number) = pre
                .rsplit_once('.')
                .and_then(|(label, n)| Some((label, n.parse::<u64>().ok()?)))
                .ok_or_else(|| invalid("a pre-release needs a numeric last part, e.g. `rc.1`"))?;
            format!("{core}-{label}.{}", number + 1)
        }
        (Bump::Breaking, None) if major == 0 => format!("0.{}.0", minor + 1),
        (Bump::Breaking, None) => format!("{}.0.0", major + 1),
    })
}

pub fn parse_version_from_asset(asset_name: &str) -> Option<String> {
    let stripped = asset_name.strip_prefix("cldr-")?;
    let end = stripped.find("-json-full")?;
    Some(stripped[..end].to_string())
}

fn workspace_cargo_toml(workspace_root: &Path) -> PathBuf {
    workspace_root.join("Cargo.toml")
}

fn locale_rs_cargo_toml(workspace_root: &Path) -> PathBuf {
    workspace_root.join("locale-rs").join("Cargo.toml")
}

fn load_doc(path: &Path) -> Result<DocumentMut> {
    Ok(fs::read_to_string(path)?.parse()?)
}

pub fn read_workspace_cldr_version(workspace_root: &Path) -> Result<Option<String>> {
    let doc = load_doc(&workspace_cargo_toml(workspace_root))?;
    Ok(doc
        .get("workspace")
        .and_then(|w| w.get("metadata"))
        .and_then(|m| m.get("cldr"))
        .and_then(|c| c.get("version"))
        .and_then(|v| v.as_str())
        .map(str::to_owned))
}

pub fn write_workspace_cldr_version(workspace_root: &Path, new_version: &str) -> Result<()> {
    let path = workspace_cargo_toml(workspace_root);
    let mut doc = load_doc(&path)?;

    let workspace = doc
        .entry("workspace")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| Error::Version("`workspace` is not a table".into()))?;
    let metadata = workspace
        .entry("metadata")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| Error::Version("`workspace.metadata` is not a table".into()))?;
    let cldr = metadata
        .entry("cldr")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| Error::Version("`workspace.metadata.cldr` is not a table".into()))?;
    cldr["version"] = value(new_version);

    fs::write(&path, doc.to_string())?;
    Ok(())
}

pub fn read_locale_rs_version(workspace_root: &Path) -> Result<String> {
    let doc = load_doc(&locale_rs_cargo_toml(workspace_root))?;
    doc.get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| Error::Version("Missing `package.version` in locale-rs/Cargo.toml".into()))
}

pub fn bump_locale_rs_version(workspace_root: &Path, bump: Bump) -> Result<(String, String)> {
    let path = locale_rs_cargo_toml(workspace_root);
    let mut doc = load_doc(&path)?;

    let current = doc
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Version("Missing `package.version` in locale-rs/Cargo.toml".into()))?
        .to_owned();
    let new = next_version(&current, bump)?;

    if new != current {
        let pkg = doc
            .get_mut("package")
            .and_then(|p| p.as_table_mut())
            .ok_or_else(|| Error::Version("`package` is not a table".into()))?;
        pkg["version"] = value(&new);
        fs::write(&path, doc.to_string())?;
    }

    Ok((current, new))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_asset_name() {
        assert_eq!(
            parse_version_from_asset("cldr-48.1.0-json-full.zip").as_deref(),
            Some("48.1.0"),
        );
        assert_eq!(parse_version_from_asset("not-a-cldr-asset.zip"), None);
    }

    #[test]
    fn breaking_bumps() {
        let next = |v| next_version(v, Bump::Breaking).unwrap();
        assert_eq!(next("0.4.2"), "0.5.0");
        assert_eq!(next("0.0.7"), "0.1.0");
        assert_eq!(next("1.2.3"), "2.0.0");
        assert_eq!(next("0.5.0-rc.1"), "0.5.0-rc.2");
        assert_eq!(next("2.0.0-beta.9"), "2.0.0-beta.10");
        assert_eq!(
            next_version("1.2.3-rc.1", Bump::None).unwrap(),
            "1.2.3-rc.1"
        );
    }

    #[test]
    fn rejects_unbumpable_versions() {
        for v in ["1.2", "1.2.beta", "1.2.3.4", "1.2.3-rc", "1.2.3-rc.x", ""] {
            assert!(next_version(v, Bump::Breaking).is_err(), "{v}");
        }
    }
}
