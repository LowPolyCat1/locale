use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

#[derive(Deserialize)]
struct GithubRelease {
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub struct CldrAsset {
    pub name: String,
    pub buffer: Vec<u8>,
}

pub fn get_latest_asset(
    output_path: &str,
) -> Result<Option<CldrAsset>, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("rust-locale-gen")
        .timeout(Duration::from_secs(300))
        .build()?;

    tracing::info!("Checking GitHub for the latest CLDR asset...");
    let release: GithubRelease = client
        .get("https://api.github.com/repos/unicode-org/cldr-json/releases/latest")
        .send()?
        .json()?;

    let asset_meta = release
        .assets
        .iter()
        .find(|a| a.name.contains("json-full.zip"))
        .ok_or("Could not find 'json-full.zip' in the latest release")?;

    if Path::new(output_path).exists() {
        let current_content = fs::read_to_string(output_path)?;
        if current_content.contains(&format!("SOURCE_ASSET: &str = \"{}\"", asset_meta.name)) {
            return Ok(None);
        }
    }

    let cache_dir = Path::new("cache");
    if !cache_dir.exists() {
        fs::create_dir(cache_dir)?;
    }
    let zip_path = cache_dir.join(&asset_meta.name);

    let buffer = if zip_path.exists() {
        tracing::info!("Using cached file: cache/{}", asset_meta.name);
        fs::read(&zip_path)?
    } else {
        tracing::info!("Downloading {}...", asset_meta.name);
        let mut response = client.get(&asset_meta.browser_download_url).send()?;
        let mut b = Vec::new();
        response.read_to_end(&mut b)?;
        fs::File::create(&zip_path)?.write_all(&b)?;
        b
    };

    Ok(Some(CldrAsset {
        name: asset_meta.name.clone(),
        buffer,
    }))
}
