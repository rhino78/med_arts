use reqwest;
use self_update::{cargo_crate_version, Status};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize)]
struct ReleaseInfo {
    tag_name: Option<String>,
}

pub fn perform_update() -> Result<Status, Box<dyn std::error::Error>> {
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

pub fn check_for_updates_blocking(
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.github.com/repos/rhino/med_arts/releases/latest")
        .header("User-Agent", "med_arts")
        .header("Authorization", "token: poop")
        .send()?;

    println!("got a response");
    if response.status() != reqwest::StatusCode::OK {
        println!("Status: {}", response.status());
        let error_text = response.text()?;
        println!("Error response: {}", error_text);
        return Err(format!("status error: {}", error_text).into());
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text()?;
        println!("HTTP Error {}: {}", status, body);
        return Err(format!("HTTP Error {}: {}", status, body).into());
    }

    let body = response.text()?;
    println!("Full response body: {}", body);

    let json: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            println!("JSON parsing Error: {}", e);
            return Err(e.into());
        }
    };

    println!("Parsed JSON: {:#?}", json);

    let latest_version_str = match json.get("tag_name") {
        Some(Value::String(s)) => s.trim_start_matches('v').to_string(),
        _ => return Err("Could not find a tag name or it's string".into()),
    };

    let current_version = Version::parse(CURRENT_VERSION)?;
    let latest_version = Version::parse(&latest_version_str)?;

    println!("Curent Version: {}", current_version);
    println!("Latest Version: {}", latest_version);

    if latest_version > current_version {
        Ok(Some(latest_version_str.to_string()))
    } else {
        Ok(None)
    }
}

pub async fn check_for_updates(
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/rhino/med_arts/releases/latest")
        .header("User-Agent", "med_arts")
        .send()
        .await?;

    let release_info: ReleaseInfo = response.json().await?;
    //let latest_version_str = release_info.tag_name.trim_start_matches("v");
    let latest_version_str = release_info
        .tag_name
        .as_ref()
        .map(|tag| tag.trim_start_matches("v").to_string())
        .unwrap_or_default();

    let current_version = Version::parse(CURRENT_VERSION)?;
    let latest_version = Version::parse(&latest_version_str)?;

    if latest_version > current_version {
        Ok(Some(latest_version_str.to_string()))
    } else {
        Ok(None)
    }
}
