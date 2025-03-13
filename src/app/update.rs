use std::env;

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
        .repo_owner("rhino78")
        .repo_name("med_arts")
        .bin_name("med_arts")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;

    Ok(status)
}

pub fn check_for_updates_blocking(
) -> Result<(Option<String>, String), Box<dyn std::error::Error + Send + Sync>> {
    match env::var("GITHUB_TOKEN") {
        Ok(s) => _ = s,
        Err(e) => {
            return Err(format!("No token: {}", e).into());
        }
    }
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.github.com/repos/rhino78/med_arts/releases/latest")
        .header("User-Agent", "med_arts/1.0 (rshave@gmail.com)")
        .header(
            "Authorization",
            format!("token {}", env::var("GITHUB_TOKEN").unwrap()),
        )
        .send()?;

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

    let json: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            println!("JSON parsing Error: {}", e);
            return Err(e.into());
        }
    };

    let release_notes = json
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or("No release notes available")
        .to_string();

    let latest_version_str = match json.get("tag_name") {
        Some(Value::String(s)) => s.trim_start_matches('v').to_string(),
        _ => return Err("Could not find a tag name or it's string".into()),
    };

    let current_version = Version::parse(CURRENT_VERSION)?;
    let latest_version = Version::parse(&latest_version_str)?;

    println!("Curent Version: {}", current_version);
    println!("Latest Version: {}", latest_version);
    println!("current notes: {}", release_notes);

    let update_available = if latest_version > current_version {
        Some(latest_version_str)
    } else {
        None
        //println!("No update available");
        //Some(latest_version_str)
    };

    Ok((update_available, release_notes))
}
