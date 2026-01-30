use locale_dev::*;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let workspace_root = find_workspace_root()?;
    let locale_rs_src = workspace_root.join("locale-rs/src");

    match download_latest::get_latest_asset()? {
        Some(asset) => {
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
        }
        None => {
            tracing::info!("Local code is already up-to-date. No action needed.");
        }
    }

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
            return Err("Could not find workspace root. Make sure you're running from within the workspace.".into());
        }
    }
}
