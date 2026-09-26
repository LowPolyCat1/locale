use locale_dev::cldr::Cldr;
use locale_dev::error::{Error, Result};
use locale_dev::policy::{DataChange, Snapshot};
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

/// Where `update_from_upstream` writes the pull request description.
const PR_BODY: &str = "target/cldr-bump-pr.md";

/// Downloads the latest CLDR release if it is newer than the recorded one,
/// regenerates the data, applies the release policy (any data change is a
/// breaking release, see `policy`), writes a pull request description to
/// `target/cldr-bump-pr.md` and syncs the READMEs.
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

    let data_dir = data_dir(workspace_root);
    let before = Snapshot::read(&data_dir)?;
    let cldr = Cldr::from_zip(asset.buffer)?;
    generate(&cldr, &new_cldr, &data_dir)?;
    let change = DataChange::between(&before, &Snapshot::read(&data_dir)?);
    log_change(&change);

    let (old_crate, new_crate) = version::bump_locale_rs_version(workspace_root, change.bump())?;
    if old_crate == new_crate {
        tracing::info!("locale-rs version unchanged ({old_crate}).");
    } else {
        tracing::info!("Breaking release: locale-rs {old_crate} -> {new_crate}");
    }

    version::write_workspace_cldr_version(workspace_root, &new_cldr)?;
    tracing::info!("workspace.metadata.cldr.version -> {new_cldr}");

    let old_cldr = current_cldr.as_deref().unwrap_or("none");
    let body = format!(
        "Automated regeneration from upstream CLDR `{new_cldr}` (was `{old_cldr}`).\n\n\
         - `locale-rs` -> `{new_crate}` (was `{old_crate}`)\n\
         {summary}\n\
         Every change to the generated data is released as a breaking change, so \
         downstream code only sees it after an explicit upgrade. Added or removed \
         locales change the exhaustive `Locale` enum.\n",
        summary = change.summary(),
    );
    let body_path = workspace_root.join(PR_BODY);
    if let Some(parent) = body_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&body_path, body)?;
    tracing::info!("Pull request description written to {PR_BODY}");

    sync_readmes(workspace_root)
}

/// Regenerates the data from a local archive without touching any version.
/// Useful offline and after changing the generator itself. If the output
/// changes, the next release must be a breaking one; that is reported, not
/// applied, since the version is up to whoever releases.
fn regenerate_from_archive(workspace_root: &Path, archive: &Path) -> Result<()> {
    let cldr_version = version::read_workspace_cldr_version(workspace_root)?.ok_or_else(|| {
        Error::Version("`--archive` needs `[workspace.metadata.cldr] version` in Cargo.toml".into())
    })?;
    tracing::info!(
        "Regenerating from {} as CLDR {cldr_version}",
        archive.display()
    );
    let data_dir = data_dir(workspace_root);
    let before = Snapshot::read(&data_dir)?;
    let cldr = Cldr::from_zip(std::fs::read(archive)?)?;
    generate(&cldr, &cldr_version, &data_dir)?;
    let change = DataChange::between(&before, &Snapshot::read(&data_dir)?);
    log_change(&change);
    if change.bump() == version::Bump::Breaking {
        tracing::warn!(
            "The generated data changed: the next locale-rs release must be a breaking one."
        );
    }
    sync_readmes(workspace_root)
}

fn log_change(change: &DataChange) {
    for line in change.summary().lines() {
        tracing::info!("{}", line.trim_start_matches("- "));
    }
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
