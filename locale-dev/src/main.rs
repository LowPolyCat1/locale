use locale_dev::cldr::Cldr;
use locale_dev::error::{Error, Result};
use locale_dev::*;
use std::path::{Path, PathBuf};

const USAGE: &str = "Usage: locale-dev [--archive <cldr-json-full.zip>] | readme [--check]";

fn main() {
    tracing_subscriber::fmt().init();
    if let Err(e) = run() {
        tracing::error!("{e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let workspace_root = find_workspace_root()?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => update_from_upstream(&workspace_root),
        ["--archive", path] => regenerate_from_archive(&workspace_root, Path::new(path)),
        ["readme"] => sync_readmes(&workspace_root),
        ["readme", "--check"] => check_readmes(&workspace_root),
        _ => Err(Error::Usage(format!("Unknown arguments {args:?}. {USAGE}"))),
    }
}

fn data_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join("locale-rs/src/data")
}

/// Downloads the latest CLDR release if it is newer than the recorded one,
/// regenerates the data, bumps the crate version and syncs the READMEs.
fn update_from_upstream(workspace_root: &Path) -> Result<()> {
    let current_cldr = version::read_workspace_cldr_version(workspace_root)?;
    match &current_cldr {
        Some(v) => tracing::info!("Current CLDR version (workspace metadata): {v}"),
        None => tracing::warn!(
            "No `[workspace.metadata.cldr] version` set; it will be recorded after generation."
        ),
    }

    let cache_dir = workspace_root.join("cache");
    let Some(asset) = download_latest::get_latest_asset(current_cldr.as_deref(), &cache_dir)?
    else {
        tracing::info!("Local code is already up-to-date. No action needed.");
        return Ok(());
    };
    let new_cldr = asset.version;

    let cldr = Cldr::from_zip(asset.buffer)?;
    generate(&cldr, &new_cldr, &data_dir(workspace_root))?;

    let bump = match current_cldr
        .as_deref()
        .and_then(version::CldrVersion::parse)
    {
        Some(old) => {
            let new = version::CldrVersion::parse(&new_cldr).ok_or_else(|| {
                Error::Version(format!("cannot parse new CLDR version `{new_cldr}`"))
            })?;
            version::classify_cldr_bump(old, new)
        }
        None => version::BumpKind::Patch,
    };

    let (old_crate, new_crate) = version::bump_locale_rs_version(workspace_root, bump)?;
    if old_crate == new_crate {
        tracing::info!("locale-rs version unchanged ({old_crate}).");
    } else {
        tracing::info!("Bumped locale-rs: {old_crate} -> {new_crate}");
    }

    version::write_workspace_cldr_version(workspace_root, &new_cldr)?;
    tracing::info!("workspace.metadata.cldr.version -> {new_cldr}");

    sync_readmes(workspace_root)
}

/// Regenerates the data from a local archive without touching any version.
/// Useful offline and after changing the generator itself.
fn regenerate_from_archive(workspace_root: &Path, archive: &Path) -> Result<()> {
    let cldr_version = version::read_workspace_cldr_version(workspace_root)?.ok_or_else(|| {
        Error::Version("`--archive` needs `[workspace.metadata.cldr] version` in Cargo.toml".into())
    })?;
    tracing::info!(
        "Regenerating from {} as CLDR {cldr_version}",
        archive.display()
    );
    let cldr = Cldr::from_zip(std::fs::read(archive)?)?;
    generate(&cldr, &cldr_version, &data_dir(workspace_root))?;
    sync_readmes(workspace_root)
}

fn sync_readmes(workspace_root: &Path) -> Result<()> {
    let updated = readme::sync(workspace_root, false)?;
    if updated.is_empty() {
        tracing::info!("READMEs are up to date.");
    }
    for file in updated {
        tracing::info!("Updated {}", file.display());
    }
    Ok(())
}

fn check_readmes(workspace_root: &Path) -> Result<()> {
    let stale = readme::sync(workspace_root, true)?;
    if stale.is_empty() {
        tracing::info!("READMEs are up to date.");
        return Ok(());
    }
    for file in &stale {
        tracing::error!("{} is out of date", file.display());
    }
    Err(Error::Readme(
        "Generated README sections are out of date. Run `cargo run -p locale-dev -- readme`."
            .into(),
    ))
}

fn find_workspace_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() && std::fs::read_to_string(&cargo_toml)?.contains("[workspace]") {
            return Ok(current);
        }
        if !current.pop() {
            return Err(Error::Usage(
                "Could not find the workspace root; run locale-dev from within the workspace."
                    .into(),
            ));
        }
    }
}
