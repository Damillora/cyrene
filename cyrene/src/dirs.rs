use std::{fs, path::PathBuf};

use directories::ProjectDirs;

use crate::errors::CyreneError;

pub struct CyreneDirs {
    pub apps_dir: PathBuf,
    pub plugins_dir: PathBuf,
    pub exe_dir: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub version_cache_path: PathBuf,
}
impl CyreneDirs {
    pub fn init_dirs(&self) -> Result<(), CyreneError> {
        fs::create_dir_all(&self.apps_dir)?;
        fs::create_dir_all(&self.plugins_dir)?;
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.cache_dir)?;

        Ok(())
    }
    pub fn installation_path(&self, name: &str, version: &str) -> PathBuf {
        let mut installation_dir = self.apps_dir.clone();
        installation_dir.push(format!("{}/{}", name, version));

        installation_dir
    }
    pub fn installation_root(&self, name: &str) -> PathBuf {
        let mut installation_dir = self.apps_dir.clone();
        installation_dir.push(name);

        installation_dir
    }
    pub fn lockfile_path(&self) -> PathBuf {
        let mut lockfile_path = self.config_dir.clone();
        lockfile_path.push("cyrene.toml");

        lockfile_path
    }
}
impl Default for CyreneDirs {
    fn default() -> Self {
        let proj_dirs = ProjectDirs::from("com", "Damillora", "Cyrene").unwrap();
        let apps_dir = match std::env::var("CYRENE_APPS_DIR") {
            Ok(env) => PathBuf::from(env),
            Err(_) => {
                let mut data_dir = proj_dirs.data_dir().to_path_buf();
                data_dir.push("apps");

                data_dir
            }
        };
        let plugins_dir = match std::env::var("CYRENE_PLUGINS_DIR") {
            Ok(env) => PathBuf::from(env),
            Err(_) => {
                let mut data_dir = proj_dirs.data_dir().to_path_buf();
                data_dir.push("plugins");

                data_dir
            }
        };
        let config_dir = proj_dirs.config_dir().to_path_buf();
        let exe_dir = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let cache_dir = proj_dirs.cache_dir().to_path_buf();
        let mut versions_cache_dir = cache_dir.clone();
        versions_cache_dir.push("versions.yaml");
        Self {
            apps_dir,
            plugins_dir,
            config_dir,
            exe_dir,
            cache_dir,
            version_cache_path: versions_cache_dir,
        }
    }
}
