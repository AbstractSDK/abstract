use reqwest::Client;
use serde::Deserialize;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use tokio::task;

#[derive(Debug, Deserialize)]
struct Release {
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

const URL: &str = "https://api.github.com/repos/AbstractSDK/contracts/releases/tags/";
const VERSION: &str = "0.15.3";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("{}v{}", URL, VERSION);
    let release: Release = client
        .get(url)
        .header("User-Agent", "request")
        .send()
        .await?
        .json()
        .await?;
    let tasks: Vec<_> = release
        .assets
        .iter()
        .filter(|asset| asset.name.ends_with(".wasm"))
        .map(|asset| {
            let name = asset.name.clone();
            let file_path = format!("../../artifacts/{name}");
            let url = asset.browser_download_url.clone();
            task::spawn(async move {
                println!("Downloading {} from {}", name, url);
                let response = reqwest::get(&url).await.unwrap();
                let path = Path::new(&file_path);
                let mut file = File::create(path).unwrap();
                let content = response.bytes().await.unwrap();
                copy(&mut content.as_ref(), &mut file).unwrap();
                println!("Downloaded {}", name);
            })
        })
        .collect();

    for task in tasks {
        task.await?;
    }

    Ok(())
}
