use log::debug;
use reqwest::header;
use rune::{ContextError, Module, Value};
use serde::Deserialize;

#[derive(Deserialize)]
struct GitHubVersion {
    tag_name: String,
    prerelease: bool,
}

#[rune::function]
fn from_github(repo: &str) -> Vec<String> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Accept", "application/vnd.github+json".parse().unwrap());
    headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());
    headers.insert("User-Agent", "damillora-cyrene".parse().unwrap());
    debug!("Getting release info from {}", repo);
    let mut versions: Vec<String> = Vec::new();
    let mut still_more_stuff = true;
    let mut page = 1;
    while still_more_stuff && page <= 10 {
        let client = reqwest::blocking::Client::new();
        debug!(
            "Calling https://api.github.com/repos/{}/releases?per_page=100&page={}",
            repo, page
        );
        let res = client
            .get(format!(
                "https://api.github.com/repos/{}/releases?per_page=100&page={}",
                repo, page
            ))
            .headers(headers.clone())
            .send()
            .unwrap();
        let a: Vec<GitHubVersion> = res.json().unwrap();
        let mut a: Vec<String> = a
            .iter()
            .filter(|f| f.prerelease == false)
            .map(|f| {
                debug!("found version: {}", f.tag_name);
                f.tag_name.to_string()
            })
            .collect();
        if a.len() < 100 {
            still_more_stuff = false;
        }
        versions.append(&mut a);
        page += 1;
    }

    versions
}

#[rune::function]
fn from_json(url: &str) -> Value {
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", "damillora-cyrene".parse().unwrap());
    debug!("Getting release info from {}", url);
    let client = reqwest::blocking::Client::new();
    debug!("Calling {}", url);
    let res = client.get(url).headers(headers.clone()).send().unwrap();
    let result: Value = res.json().unwrap();

    result
}

pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_crate("versions")?;
    m.function_meta(from_github)?;
    m.function_meta(from_json)?;
    Ok(m)
}
