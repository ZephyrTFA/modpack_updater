#![allow(unused)]

use std::{any::Any, env::args, error::Error, future::IntoFuture, iter::Map};

use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;

fn main() -> Result<(), String> {
    let args = args().collect::<Vec<_>>();
    if args.len() != 4 {
        println!("Must specify the github owner, github repo, and server file regex and nothing else.");
        return Ok(());
    }

    let regex = Regex::new(args.get(3).unwrap()).map_err(|e| e.to_string())?;
    for asset in
        get_latest_json(&format!("https://api.github.com/repos/{}/{}/releases/latest", args.get(1).unwrap(), args.get(2).unwrap()))
            .map(|v| v.assets)?
    {
        let captures = regex.captures(&asset.name);
        if captures.is_none() {
            #[cfg(debug_assertions)]
            println!("no match: {}", &asset.name);
            continue;
        }

        let captures = captures.unwrap();
        println!(
            "{}{}",
            if let Some(found) = captures.get(1) {
                format!("{}|", found.as_str())
            } else {
                "".to_string()
            },
            asset.browser_download_url
        );
        return Ok(());
    }

    return Err("Failed to determine correct asset.".into());
}

#[derive(Deserialize)]
struct ReleaseAsset {
    pub browser_download_url: String,
    pub name: String,
}

#[derive(Deserialize)]
struct Release {
    pub name: String,
    pub assets: Vec<ReleaseAsset>,
}

fn get_latest_json(url: &str) -> Result<Release, String> {
    let client = Client::default();

    let response = client
        .get(url)
        .header("User-Agent", "ModpackUpdater 1.0.0")
        .send()
        .map_err(|e| e.to_string())?;
    let body = response.text().map_err(|e| e.to_string())?;
    let json = body.lines().next().unwrap();
    serde_json::from_str(json).map_err(|e| e.to_string())
}
