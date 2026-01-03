use locale_dev::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let output_path = "./locale/src/locale.rs";

    match download_latest::get_latest_asset(output_path)? {
        Some(asset) => {
            generate_locales::run(asset.buffer, &asset.name, output_path)?;
        }
        None => {
            tracing::info!("Local code is already up-to-date. No action needed.");
        }
    }

    Ok(())
}
