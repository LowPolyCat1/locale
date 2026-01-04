use locale_dev::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    match download_latest::get_latest_asset()? {
        Some(asset) => {
            generate_locales::run(
                asset.buffer.clone(),
                &asset.name,
                "./locale-rs/src/locale.rs",
            )?;
            generate_num_formats::run(
                asset.buffer.clone(),
                &asset.name,
                "./locale-rs/src/num_formats.rs",
            )?;
            generate_datetime_formatting::run(
                asset.buffer.clone(),
                &asset.name,
                "./locale-rs/src/datetime_formats.rs",
            )?;
            format::format_generated_code();
        }
        None => {
            tracing::info!("Local code is already up-to-date. No action needed.");
        }
    }

    Ok(())
}
