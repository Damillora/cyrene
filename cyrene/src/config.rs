use std::{fs, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

use crate::errors::CyreneError;


#[derive(Serialize, Deserialize)]
pub struct CyreneConfig {
    pub apps_dir: Option<PathBuf>,
    pub plugins_dir: Option<PathBuf>,
    pub install_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
    pub lockfile_path: Option<PathBuf>,
}

impl CyreneConfig {
    pub fn load(config_path: &Path) -> Result<CyreneConfig, CyreneError> {
        let config = if !fs::exists(config_path).map_err(CyreneError::ConfigRead)? {
            let config = CyreneConfig {
                apps_dir: None,
                plugins_dir: None,
                install_dir: None,
                cache_dir: None,
                lockfile_path: None,
            };
            let config_toml = toml::ser::to_string(&config).map_err(CyreneError::ConfigSerialize)?;
            fs::write(config_path, config_toml).map_err(CyreneError::ConfigWrite)?;

            config
        } else {
            let config_read =
                fs::read_to_string(config_path).map_err(CyreneError::ConfigRead)?;
            let config: CyreneConfig =
                toml::de::from_str(&config_read).map_err(CyreneError::ConfigDeserialize)?;
            config
        };

        Ok(config)
    }
}