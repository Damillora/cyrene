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
    let client = reqwest::blocking::Client::new();
    let res = client
        .get(format!("https://api.github.com/repos/{}/releases", repo))
        .headers(headers)
        .send()
        .unwrap();
    let a: Vec<GitHubVersion> = res.json().unwrap();

    a.iter().map(|f| f.tag_name.clone()).collect()
}

pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_crate("versions")?;
    m.function_meta(from_github)?;
    Ok(m)
}
