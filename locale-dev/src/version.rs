use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Table, value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CldrVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl CldrVersion {
    pub fn parse(s: &str) -> Option<Self> {
        let mut parts = s.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

impl std::fmt::Display for CldrVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpKind {
    Major,
    Minor,
    Patch,
    None,
}

pub fn classify_cldr_bump(old: CldrVersion, new: CldrVersion) -> BumpKind {
    if new.major != old.major {
        BumpKind::Major
    } else if new.minor != old.minor {
        BumpKind::Minor
    } else if new.patch != old.patch {
        BumpKind::Patch
    } else {
        BumpKind::None
    }
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

fn load_doc(path: &Path) -> Result<DocumentMut, Box<dyn std::error::Error>> {
    Ok(fs::read_to_string(path)?.parse()?)
}

pub fn read_workspace_cldr_version(
    workspace_root: &Path,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let doc = load_doc(&workspace_cargo_toml(workspace_root))?;
    Ok(doc
        .get("workspace")
        .and_then(|w| w.get("metadata"))
        .and_then(|m| m.get("cldr"))
        .and_then(|c| c.get("version"))
        .and_then(|v| v.as_str())
        .map(str::to_owned))
}

pub fn write_workspace_cldr_version(
    workspace_root: &Path,
    new_version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = workspace_cargo_toml(workspace_root);
    let mut doc = load_doc(&path)?;

    let workspace = doc
        .entry("workspace")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .ok_or("`workspace` is not a table")?;
    let metadata = workspace
        .entry("metadata")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .ok_or("`workspace.metadata` is not a table")?;
    let cldr = metadata
        .entry("cldr")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .ok_or("`workspace.metadata.cldr` is not a table")?;
    cldr["version"] = value(new_version);

    fs::write(&path, doc.to_string())?;
    Ok(())
}

pub fn read_locale_rs_version(workspace_root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let doc = load_doc(&locale_rs_cargo_toml(workspace_root))?;
    doc.get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| "Missing `package.version` in locale-rs/Cargo.toml".into())
}

pub fn bump_locale_rs_version(
    workspace_root: &Path,
    bump: BumpKind,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let path = locale_rs_cargo_toml(workspace_root);
    let mut doc = load_doc(&path)?;

    let current = doc
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or("Missing `package.version` in locale-rs/Cargo.toml")?
        .to_owned();

    let parts: Vec<u32> = current
        .split('.')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|e| format!("Cannot parse locale-rs version `{current}`: {e}"))?;
    if parts.len() != 3 {
        return Err(format!("Expected MAJOR.MINOR.PATCH in `{current}`").into());
    }
    let (maj, min, pat) = (parts[0], parts[1], parts[2]);

    let new = match (maj, bump) {
        (_, BumpKind::None) => current.clone(),
        (0, BumpKind::Major | BumpKind::Minor) => format!("0.{}.0", min + 1),
        (0, BumpKind::Patch) => format!("0.{}.{}", min, pat + 1),
        (_, BumpKind::Major) => format!("{}.0.0", maj + 1),
        (_, BumpKind::Minor) => format!("{}.{}.0", maj, min + 1),
        (_, BumpKind::Patch) => format!("{}.{}.{}", maj, min, pat + 1),
    };

    if new != current {
        let pkg = doc
            .get_mut("package")
            .and_then(|p| p.as_table_mut())
            .ok_or("`package` is not a table")?;
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
    fn classifies_bumps() {
        let v = |s| CldrVersion::parse(s).unwrap();
        assert_eq!(
            classify_cldr_bump(v("48.1.0"), v("49.0.0")),
            BumpKind::Major
        );
        assert_eq!(
            classify_cldr_bump(v("48.0.0"), v("48.1.0")),
            BumpKind::Minor
        );
        assert_eq!(
            classify_cldr_bump(v("48.1.0"), v("48.1.1")),
            BumpKind::Patch
        );
        assert_eq!(classify_cldr_bump(v("48.1.0"), v("48.1.0")), BumpKind::None);
    }
}
