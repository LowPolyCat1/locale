use locale_dev::*;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let workspace_root = find_workspace_root()?;
    let locale_rs_src = workspace_root.join("locale-rs/src");

    let current_cldr = version::read_workspace_cldr_version(&workspace_root)?;
    match &current_cldr {
        Some(v) => tracing::info!("Current CLDR version (workspace metadata): {v}"),
        None => tracing::warn!(
            "No `[workspace.metadata.cldr] version` set — will populate after first generation."
        ),
    }

    let asset = match download_latest::get_latest_asset(current_cldr.as_deref())? {
        Some(asset) => asset,
        None => {
            tracing::info!("Local code is already up-to-date. No action needed.");
            return Ok(());
        }
    };
    let new_cldr = asset.version.clone();

    generate_locales::run(
        asset.buffer.clone(),
        &asset.name,
        locale_rs_src.join("locale.rs").to_str().unwrap(),
    )?;
    generate_num_formats::run(
        asset.buffer.clone(),
        &asset.name,
        locale_rs_src.join("num_formats.rs").to_str().unwrap(),
    )?;
    generate_datetime_formatting::run(
        asset.buffer.clone(),
        &asset.name,
        locale_rs_src.join("datetime_formats.rs").to_str().unwrap(),
    )?;
    generate_currency_formatting::run(
        asset.buffer.clone(),
        &asset.name,
        locale_rs_src.join("currency_formats.rs").to_str().unwrap(),
    )?;
    format::format_generated_code();

    let bump = match current_cldr
        .as_deref()
        .and_then(version::CldrVersion::parse)
    {
        Some(old) => {
            let new = version::CldrVersion::parse(&new_cldr)
                .ok_or_else(|| format!("Failed to parse new CLDR version `{new_cldr}`"))?;
            version::classify_cldr_bump(old, new)
        }
        None => version::BumpKind::Patch,
    };

    let (old_crate, new_crate) = version::bump_locale_rs_version(&workspace_root, bump)?;
    if old_crate == new_crate {
        tracing::info!("locale-rs version unchanged ({old_crate}).");
    } else {
        tracing::info!("Bumped locale-rs: {old_crate} -> {new_crate}");
    }

    version::write_workspace_cldr_version(&workspace_root, &new_cldr)?;
    tracing::info!("workspace.metadata.cldr.version -> {new_cldr}");

    Ok(())
}

fn find_workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut current = std::env::current_dir()?;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(current);
            }
        }

        if !current.pop() {
            return Err(
                "Could not find workspace root. Make sure you're running from within the workspace."
                    .into(),
            );
        }
    }
}
