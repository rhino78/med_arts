use self_update::{cargo_crate_version, Status};
use semver::Version;
use reqwest;
use serde::Deserialize;

#[derive(Deserialize)]
struct ReleaseInfo {
    tag_name: String,
}

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn perform_update() ->Result<Status, Box<dyn std::error::Error>>{
    let status = self_update::backends::github::Update::configure()
        .repo_owner("rhino")
        .repo_name("med_arts")
        .bin_name("med_arts")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    Ok(status)
}

pub fn check_for_updates_blocking() -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {

    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.github.com/repos/rhino/med_arts/releases/latest")
        .header("User-Agent", "med_arts")
        .send()?;

    let release_info: ReleaseInfo = response.json()?;
    let latest_version_str = release_info.tag_name.trim_start_matches("v");
    let current_version = Version::parse(CURRENT_VERSION)?;
    let latest_version = Version::parse(latest_version_str)?;

    if latest_version > current_version {
        Ok(Some(latest_version_str.to_string()))
    } else {
        Ok(None)
    }
}

pub async fn check_for_updates() -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/rhino/med_arts/releases/latest")
        .header("User-Agent", "med_arts")
        .send()
        .await?;

    let release_info: ReleaseInfo = response.json().await?;
    let latest_version_str = release_info.tag_name.trim_start_matches("v");
    let current_version = Version::parse(CURRENT_VERSION)?;
    let latest_version = Version::parse(latest_version_str)?;

    if latest_version > current_version {
        Ok(Some(latest_version_str.to_string()))
    } else {
        Ok(None)
    }
}
