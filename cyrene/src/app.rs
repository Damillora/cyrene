use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use text_template::Template;
use url::Url;

use crate::{
    app_module::{post_install::process_post_install, sources::process_source, versions},
    errors::CyreneError,
};

#[derive(Serialize, Deserialize)]
pub struct CyreneApp {
    pub settings: AppSettings,
    pub versions: AppVersions,
    pub sources: Vec<AppSources>,
    pub binaries: HashMap<String, String>,
    pub post_install: Option<Vec<AppPostInstallCommands>>,
}
#[derive(Serialize, Deserialize)]
pub struct AppSettings {
    pub upgrade_latest: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AppVersions {
    Github {
        repo: String,
        command: Option<Vec<AppVersionsGithubCommand>>,
    },
    Url {
        url: Url,
        command: Vec<AppVersionsUrlCommand>,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppVersionsGithubCommand {
    StripPrefix { prefix: String },
    Replace { str: String, with: String},
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppVersionsUrlCommand {
    Jsonpath { query: String },
    StripPrefix { prefix: String },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppSources {
    TarXz { url: String },
    TarGz { url: String },
    Zip { url: String },
    File { url: String },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppPostInstallCommands {
    SetExec { path: String },
}

// Instance functions
impl CyreneApp {
    pub async fn get_versions(&self) -> Result<Vec<String>, CyreneError> {
        versions::process_version(&self.versions).await
    }

    pub async fn install(&self, version: &str, installation_dir: &Path) -> Result<(), CyreneError> {
        for source in &self.sources {
            process_source(source, version, installation_dir).await?;
        }

        Ok(())
    }

    pub async fn post_install(
        &self,
        version: &str,
        installation_dir: &Path,
    ) -> Result<(), CyreneError> {
        let mut values = HashMap::new();
        values.insert("version", version);
        if let Some(post_install) = &self.post_install {
            for post_install in post_install {
                process_post_install(post_install, version, installation_dir).await?;
            }
        }

        Ok(())
    }

    pub fn binaries(&self, version: &str) -> Result<HashMap<String, String>, CyreneError> {
        let mut values = HashMap::new();
        values.insert("version", version);
        let new_map = self
            .binaries
            .clone()
            .into_iter()
            .map(|(key, value)| {
                let tmpl = Template::from(value.as_str());
                let new_val = tmpl.fill_in(&values);
                (key, new_val.to_string())
            })
            .collect();
        Ok(new_map)
    }

    pub fn upgrade_latest(&self) -> bool {
        todo!()
    }
}

impl CyreneApp {
    pub fn from_str(config: &str) -> Result<CyreneApp, CyreneError> {
        let app: CyreneApp = toml::de::from_str(config).map_err(CyreneError::AppDeserialize)?;

        Ok(app)
    }
    pub fn from_file(path: &Path) -> Result<CyreneApp, CyreneError> {
        let file =
            fs::read_to_string(path).map_err(|e| CyreneError::AppRead(path.to_path_buf(), e))?;

        CyreneApp::from_str(&file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_app() {
        let config = r#"
[settings]
upgrade_latest = false

[versions]
type = "github"
repo = "Damillora/cyrene"

[[sources]]
type = "tar_xz"
url = "https://github.com/Damillora/cyrene/releases/download/${env.version}/cyrene-x86_64-unknown-linux-gnu.tar.xz"

[binaries]
cyrene = "cyrene-x86_64-unknown-linux-gnu/cyrene"
"#;
        let app: CyreneApp = toml::de::from_str(&config).unwrap();
        if let AppVersions::Github { repo } = app.versions {
            assert_eq!(repo, "Damillora/cyrene");
        } else {
            panic!("Not GitHub source");
        }
        assert_eq!(app.sources.len(), 1);
        assert_eq!(app.binaries.len(), 1);
    }

    #[test]
    fn custom_app() {
        let config = r#"
[settings]
upgrade_latest = false

[versions]
type = "url"
url = "https://nodejs.org/dist/index.json"

[versions.command]
type = "jsonpath"
query = "$[*].version"

[[sources]]
type = "tar_xz"
url = "https://nodejs.org/dist/v${version}/node-v${version}-linux-x64.tar.xz"

[binaries]
node = "node-v${version}-linux-x64/bin/node"
"#;
        let app: CyreneApp = toml::de::from_str(&config).unwrap();

        if let AppVersions::Url { url, command } = app.versions {
            assert_eq!(url.as_str(), "https://nodejs.org/dist/index.json");
        } else {
            panic!("Not URL source");
        }
        assert_eq!(app.sources.len(), 1);
        assert_eq!(app.binaries.len(), 1);
    }
}
