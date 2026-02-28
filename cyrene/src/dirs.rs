use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use log::debug;

use crate::{config::CyreneConfig, errors::CyreneError};

pub struct CyreneDirs {
    pub apps_dir: PathBuf,
    pub plugins_dir: PathBuf,
    pub exe_dir: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub version_cache_path: PathBuf,
    lockfile_path: PathBuf,
}
impl CyreneDirs {
    pub fn init_dirs(&self) -> Result<(), CyreneError> {
        debug!("Creating {}", &self.apps_dir.display());
        fs::create_dir_all(&self.apps_dir)
            .map_err(|e| CyreneError::DirectoryInit(self.apps_dir.clone(), e))?;
        debug!("Creating {}", &self.plugins_dir.display());
        fs::create_dir_all(&self.plugins_dir)
            .map_err(|e| CyreneError::DirectoryInit(self.plugins_dir.clone(), e))?;
        debug!("Creating {}", &self.config_dir.display());
        fs::create_dir_all(&self.config_dir)
            .map_err(|e| CyreneError::DirectoryInit(self.apps_dir.clone(), e))?;
        debug!("Creating {}", &self.cache_dir.display());
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| CyreneError::DirectoryInit(self.apps_dir.clone(), e))?;
        debug!("Creating {}", &self.exe_dir.display());
        fs::create_dir_all(&self.exe_dir)
            .map_err(|e| CyreneError::DirectoryInit(self.apps_dir.clone(), e))?;

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
        self.lockfile_path.clone()
    }

    pub fn ensure_installation_dir(
        &self,
        name: &str,
        version: &str,
    ) -> Result<PathBuf, CyreneError> {
        // $CYRENE_APPS_DIR/app_name-app_version
        let installation_path = self.installation_path(name, version.to_string().as_str());
        fs::create_dir_all(&installation_path).map_err(|e| {
            CyreneError::AppInstallDirCreateError(name.to_string(), version.to_string(), e)
        })?;

        Ok(installation_path)
    }
}
impl CyreneDirs {
    pub fn new(config_path: &Option<String>) -> Result<Self, CyreneError> {
        let proj_dirs = ProjectDirs::from("com", "Damillora", "Cyrene").unwrap();

        let config_path = if let Some(conf) = config_path {
            PathBuf::from(conf)
        } else {
            let mut config_path = proj_dirs.config_dir().to_path_buf();
            config_path.push("cyrene.toml");

            config_path
        };
        let config = CyreneConfig::load(&config_path)?;
        let apps_dir = match std::env::var("CYRENE_APPS_DIR") {
            Ok(env) => PathBuf::from(env),
            Err(_) => {
                if let Some(apps_dir) = &config.apps_dir {
                    apps_dir.clone()
                } else {
                    let mut data_dir = proj_dirs.data_dir().to_path_buf();
                    data_dir.push("apps");

                    data_dir
                }
            }
        };
        let plugins_dir = match std::env::var("CYRENE_PLUGINS_DIR") {
            Ok(env) => PathBuf::from(env),
            Err(_) => {
                if let Some(plugins_dir) = &config.plugins_dir {
                    plugins_dir.clone()
                } else {
                    let mut data_dir = proj_dirs.data_dir().to_path_buf();
                    data_dir.push("plugins");

                    data_dir
                }
            }
        };
        let config_dir = proj_dirs.config_dir().to_path_buf();
        let exe_dir = match std::env::var("CYRENE_INSTALL_DIR") {
            Ok(env) => PathBuf::from(env),
            Err(_) => {
                if let Some(install_dir) = &config.install_dir {
                    install_dir.clone()
                } else {
                    std::env::current_exe()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .to_path_buf()
                }
            }
        };
        let cache_dir = if let Some(cache_dir) = &config.cache_dir {
            cache_dir.clone()
        } else {
            proj_dirs.cache_dir().to_path_buf()
        };
        let mut versions_cache_dir = cache_dir.clone();
        versions_cache_dir.push("versions.yaml");
        let lockfile_path = if let Some(lockfile_path) = &config.lockfile_path {
            lockfile_path.clone()
        } else {
            let mut lockfile_path = config_dir.clone();
            lockfile_path.push("cyrene.lock");

            lockfile_path
        };
        Ok(Self {
            apps_dir,
            plugins_dir,
            config_dir,
            exe_dir,
            cache_dir,
            version_cache_path: versions_cache_dir,
            lockfile_path,
        })
    }
}
