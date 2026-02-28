use jsonpath_rust::JsonPath;
use log::debug;
use regex::Regex;
use reqwest::header;
use serde::Deserialize;
use serde_json::Value;
use url::Url;

use crate::{
    app::{AppVersions, AppVersionsGithubCommand, AppVersionsUrlCommand},
    errors::CyreneError,
};

#[derive(Deserialize)]
struct GitHubVersion {
    tag_name: String,
    prerelease: bool,
}

fn sanitize_version(ver: &str, ver_regex: &Regex) -> String {
    let mut version = ver.to_string();
    if let Some(captures) = ver_regex.captures(&version)
        && let Some(ver_name) = captures.get(2)
    {
        version = String::from(ver_name.as_str())
    } else if version.starts_with("v") {
        version = version.trim_start_matches("v").to_string();
    }

    version
}

async fn process_github(
    repo: &str,
    command: &Option<Vec<AppVersionsGithubCommand>>,
) -> Result<Vec<String>, CyreneError> {
    let ver_regex = Regex::new(r"(.*)-v?([0-9\.]*)").unwrap();

    let mut headers = header::HeaderMap::new();
    headers.insert("Accept", "application/vnd.github+json".parse().unwrap());
    headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());
    headers.insert("User-Agent", "damillora-cyrene".parse().unwrap());
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        debug!("Found GitHub token");
        headers.insert("Authorization", format!("Bearer {}", token).parse().unwrap());
    }
    debug!("Getting release info from {}", repo);
    let mut versions: Vec<String> = Vec::new();
    let mut still_more_stuff = true;
    let mut page = 1;

    while still_more_stuff && page <= 10 {
        let client = reqwest::Client::new();
        debug!(
            "Calling https://api.github.com/repos/{}/releases?per_page=100&page={}",
            repo, page
        );
        let url = format!(
            "https://api.github.com/repos/{}/releases?per_page=100&page={}",
            repo, page
        );
        let res = client
            .get(&url)
            .headers(headers.clone())
            .send()
            .await
            .map_err(|e| CyreneError::VersionFetch(url.clone(), e))?;
        let res = res
            .error_for_status()
            .map_err(|e| CyreneError::VersionFetch(url.clone(), e))?;
        let a: Vec<GitHubVersion> = res
            .json()
            .await
            .map_err(|e| CyreneError::VersionFetch(url.clone(), e))?;
        let mut a: Vec<String> = a
            .iter()
            .filter(|f| !f.prerelease)
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
    if let Some(command) = command {
        for command in command {
            match command {
                AppVersionsGithubCommand::StripPrefix { prefix } => {
                    versions = versions
                        .iter()
                        .map(|e| e.strip_prefix(prefix).unwrap_or("").to_string())
                        .collect();
                }
                AppVersionsGithubCommand::Replace { str, with } => {
                    versions = versions.iter().map(|e| e.replace(str, with)).collect();
                }
            }
        }
    }
    Ok(versions)
}

async fn process_url(
    url: &Url,
    command: &Vec<AppVersionsUrlCommand>,
) -> Result<Vec<String>, CyreneError> {
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", "damillora-cyrene".parse().unwrap());
    debug!("Getting release info from {}", url);
    let client = reqwest::Client::new();
    debug!("Calling {}", url);
    let res = client
        .get(url.to_string())
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| CyreneError::VersionFetch(url.to_string(), e))?;
    let result: Value = res
        .json()
        .await
        .map_err(|e| CyreneError::VersionFetch(url.to_string(), e))?;
    let mut results: Vec<String> = Vec::new();
    for command in command {
        match command {
            AppVersionsUrlCommand::Jsonpath { query } => {
                let processed_value: Vec<&Value> = result
                    .query(query)
                    .map_err(|e| CyreneError::VersionQueryParse(query.clone(), e))?;
                let processed_value: Vec<String> = processed_value
                    .iter()
                    .filter_map(|f| f.as_str())
                    .map(|f| f.to_string())
                    .collect();
                results = processed_value;
            }
            AppVersionsUrlCommand::StripPrefix { prefix } => {
                results = results
                    .iter()
                    .map(|e| e.strip_prefix(prefix).unwrap_or("").to_string())
                    .collect();
            }
        }
    }
    Ok(results)
}

pub async fn process_version(versions: &AppVersions) -> Result<Vec<String>, CyreneError> {
    match versions {
        AppVersions::Github { repo, command } => process_github(repo, command).await,
        AppVersions::Url { url, command } => process_url(url, command).await,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn test_github() {
        let version = AppVersions::Github {
            repo: "Damillora/cyrene".to_string(),
            command: None,
        };
        let result = process_version(&version).await;

        if let Ok(result) = result {
            assert!(result.len() > 0);
            println!("{:?}", result);
            assert!(result.iter().any(|f| f.eq("0.3.0")));
        } else {
            panic!("Not ok");
        }
    }

    #[tokio::test]
    async fn test_custom() {
        let version = AppVersions::Url {
            url: Url::from_str("https://nodejs.org/dist/index.json").unwrap(),
            command: vec![AppVersionsUrlCommand::Jsonpath {
                query: "$[*].version".to_string(),
            }],
        };

        let result = process_version(&version).await;

        if let Ok(result) = result {
            assert!(result.len() > 0);
            println!("{:?}", result);
            assert!(result.iter().any(|f| f.eq("22.0.0")));
        } else {
            panic!("Not ok");
        }
    }
}
