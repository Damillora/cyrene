use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::{errors::CyreneError, responses::CyreneAppItem};

#[derive(Default, Serialize, Deserialize)]
pub struct CyreneLockfile {
    pub versions: BTreeMap<String, String>,
    pub loaded_lockfile: Option<String>,
}

pub struct CyreneLockfileManager {
    lockfile_path: PathBuf,
}
impl CyreneLockfileManager {
    pub fn new(lockfile_path: &Path) -> Self {
        Self {
            lockfile_path: lockfile_path.to_path_buf(),
        }
    }
    pub fn find_installed_version_from_lockfile(
        &self,
        name: &str,
    ) -> Result<Option<String>, CyreneError> {
        let mut lockfile = if !fs::exists(&self.lockfile_path)? {
            CyreneLockfile::default()
        } else {
            let lockfile_read = fs::read_to_string(&self.lockfile_path)?;
            let lockfile: CyreneLockfile = toml::de::from_str(&lockfile_read)?;
            lockfile
        };
        if let Some(loaded_lockfile) = lockfile.loaded_lockfile {
            // Merge global lockfile with local ones
            let new_lockfile = {
                let lockfile_read = fs::read_to_string(&loaded_lockfile)?;
                let new_lockfile: CyreneLockfile = toml::de::from_str(&lockfile_read)?;

                new_lockfile
            };
            for (key, value) in new_lockfile.versions {
                lockfile.versions.insert(key, value);
            }
        }
        let version = lockfile.versions.get(name).map(|x| x.to_string());
        Ok(version)
    }

    pub fn update_lockfile(&self, name: &str, version: &str) -> Result<(), CyreneError> {
        let mut lockfile_path = PathBuf::from(&self.lockfile_path);
        let mut lockfile = if !fs::exists(&lockfile_path)? {
            CyreneLockfile::default()
        } else {
            let lockfile_read = fs::read_to_string(&lockfile_path)?;
            let lockfile: CyreneLockfile = toml::de::from_str(&lockfile_read)?;
            lockfile
        };
        if let Some(loaded_lockfile) = lockfile.loaded_lockfile {
            // Save changes to new lockfile
            lockfile = {
                lockfile_path = PathBuf::from(&loaded_lockfile);
                let lockfile_read = fs::read_to_string(&loaded_lockfile)?;
                let lockfile: CyreneLockfile = toml::de::from_str(&lockfile_read)?;
                lockfile
            }
        }
        debug!("Using lockfile {}", lockfile_path.to_string_lossy());
        lockfile
            .versions
            .insert(name.to_owned(), version.to_owned());
        let lockfile_write = toml::ser::to_string(&lockfile)?;
        fs::write(lockfile_path, lockfile_write)?;
        Ok(())
    }

    pub fn use_default_lockfile(&self) -> Result<(), CyreneError> {
        let mut lockfile = if !fs::exists(&self.lockfile_path)? {
            CyreneLockfile::default()
        } else {
            let lockfile_read = fs::read_to_string(&self.lockfile_path)?;
            let lockfile: CyreneLockfile = toml::de::from_str(&lockfile_read)?;
            lockfile
        };
        lockfile.loaded_lockfile = None;
        let lockfile_write = toml::ser::to_string(&lockfile)?;
        fs::write(&self.lockfile_path, lockfile_write)?;
        Ok(())
    }

    pub fn use_local_lockfile(&self, loaded_lockfile: &Path) -> Result<(), CyreneError> {
        let mut lockfile = if !fs::exists(&self.lockfile_path)? {
            CyreneLockfile::default()
        } else {
            let lockfile_read = fs::read_to_string(&self.lockfile_path)?;
            let lockfile: CyreneLockfile = toml::de::from_str(&lockfile_read)?;
            lockfile
        };
        lockfile.loaded_lockfile = Some(
            fs::canonicalize(loaded_lockfile)?
                .to_string_lossy()
                .to_string(),
        );
        let lockfile_write = toml::ser::to_string(&lockfile)?;
        fs::write(&self.lockfile_path, lockfile_write)?;
        Ok(())
    }

    pub fn is_local_lockfile(&self) -> Result<bool, CyreneError> {
        let lockfile = if !fs::exists(&self.lockfile_path)? {
            CyreneLockfile::default()
        } else {
            let lockfile_read = fs::read_to_string(&self.lockfile_path)?;
            let lockfile: CyreneLockfile = toml::de::from_str(&lockfile_read)?;
            lockfile
        };
        Ok(lockfile.loaded_lockfile.is_some())
    }

    pub fn load_versions_from_current_lockfile(&self) -> Result<Vec<CyreneAppItem>, CyreneError> {
        let mut lockfile = if !fs::exists(&self.lockfile_path)? {
            CyreneLockfile::default()
        } else {
            let lockfile_read = fs::read_to_string(&self.lockfile_path)?;
            let lockfile: CyreneLockfile = toml::de::from_str(&lockfile_read)?;
            lockfile
        };
        if let Some(loaded_lockfile) = lockfile.loaded_lockfile {
            // Load needed versions from new lockfile
            lockfile = {
                let lockfile_read = fs::read_to_string(&loaded_lockfile)?;
                let lockfile: CyreneLockfile = toml::de::from_str(&lockfile_read)?;
                lockfile
            };
        }
        let version: Vec<_> = lockfile
            .versions
            .into_iter()
            .map(|(key, value)| CyreneAppItem {
                name: key,
                version: value,
            })
            .collect();
        Ok(version)
    }
}
