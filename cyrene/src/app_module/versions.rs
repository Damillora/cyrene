use log::debug;
use reqwest::header;
use rune::{ContextError, Module};
use serde::Deserialize;

#[derive(Deserialize)]
struct GitHubVersion {
    tag_name: String,
}

#[rune::function]
fn from_github(repo: String) -> Vec<String> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Accept", "application/vnd.github+json".parse().unwrap());
    headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());
    headers.insert("User-Agent", "damillora-cyrene".parse().unwrap());
    debug!("Getting release info from {}", repo);
    let mut versions: Vec<String> = Vec::new();
    let mut still_more_stuff = true;
    while still_more_stuff {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!(
                "https://api.github.com/repos/{}/releases?per_page=100",
                repo
            ))
            .headers(headers.clone())
            .send()
            .unwrap();
        let a: Vec<GitHubVersion> = res.json().unwrap();
        let mut a: Vec<String> = a
            .iter()
            .map(|f| f.tag_name.trim_start_matches("v").to_string())
            .collect();
        if a.len() < 100 {
            still_more_stuff = false;
        }
        versions.append(&mut a);
    }

    versions
}

pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_crate("versions")?;
    m.function_meta(from_github)?;
    Ok(m)
}
