use locale_dev::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    match download_latest::get_latest_asset()? {
        Some(asset) => {
            generate_locales::run(asset.buffer.clone(), &asset.name, "./locale/src/locale.rs")?;
            generate_num_formats::run(
                asset.buffer.clone(),
                &asset.name,
                "./locale/src/num_formats.rs",
            )?;
            generate_datetime_formatting::run(
                asset.buffer.clone(),
                &asset.name,
                "./locale/src/datetime_formats.rs",
            )?;
        }
        None => {
            tracing::info!("Local code is already up-to-date. No action needed.");
        }
    }

    Ok(())
}
