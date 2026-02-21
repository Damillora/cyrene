use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::errors::CyreneError;

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
        let mut lockfile = if !fs::exists(&self.lockfile_path).map_err(CyreneError::LockfileRead)? {
            CyreneLockfile::default()
        } else {
            let lockfile_read =
                fs::read_to_string(&self.lockfile_path).map_err(CyreneError::LockfileRead)?;
            let lockfile: CyreneLockfile =
                toml::de::from_str(&lockfile_read).map_err(CyreneError::LockfileDeserialize)?;
            lockfile
        };
        if let Some(loaded_lockfile) = lockfile.loaded_lockfile {
            // Merge global lockfile with local ones
            let new_lockfile = {
                let lockfile_read = fs::read_to_string(&loaded_lockfile).map_err(|e| {
                    CyreneError::LockfileLocalRead(PathBuf::from(loaded_lockfile), e)
                })?;
                let new_lockfile: CyreneLockfile =
                    toml::de::from_str(&lockfile_read).map_err(CyreneError::LockfileDeserialize)?;

                new_lockfile
            };
            for (key, value) in new_lockfile.versions {
                lockfile.versions.insert(key, value);
            }
        }
        let version = lockfile.versions.get(name).map(|x| x.to_string());
        debug!("lockfile found app {} version {:?}", &name, &version);
        Ok(version)
    }

    pub fn update_lockfile(&self, name: &str, version: Option<&str>) -> Result<(), CyreneError> {
        let mut lockfile_path = PathBuf::from(&self.lockfile_path);
        let mut lockfile = if !fs::exists(&lockfile_path).map_err(CyreneError::LockfileRead)? {
            CyreneLockfile::default()
        } else {
            let lockfile_read =
                fs::read_to_string(&lockfile_path).map_err(CyreneError::LockfileRead)?;
            let lockfile: CyreneLockfile =
                toml::de::from_str(&lockfile_read).map_err(CyreneError::LockfileDeserialize)?;
            lockfile
        };
        if let Some(loaded_lockfile) = lockfile.loaded_lockfile {
            // Save changes to new lockfile
            lockfile = {
                lockfile_path = PathBuf::from(&loaded_lockfile);
                let lockfile_read = fs::read_to_string(&loaded_lockfile).map_err(|e| {
                    CyreneError::LockfileLocalRead(PathBuf::from(&loaded_lockfile), e)
                })?;
                let lockfile: CyreneLockfile =
                    toml::de::from_str(&lockfile_read).map_err(CyreneError::LockfileDeserialize)?;
                lockfile
            }
        }
        debug!("Using lockfile {}", lockfile_path.to_string_lossy());
        if let Some(version) = version {
            lockfile
                .versions
                .insert(name.to_owned(), version.to_owned());
        } else {
            lockfile.versions.remove(name);
        }
        let lockfile_write =
            toml::ser::to_string(&lockfile).map_err(CyreneError::LockfileSerialize)?;
        fs::write(lockfile_path, lockfile_write).map_err(CyreneError::LockfileWrite)?;
        Ok(())
    }

    pub fn use_default_lockfile(&self) -> Result<(), CyreneError> {
        let mut lockfile = if !fs::exists(&self.lockfile_path).map_err(CyreneError::LockfileRead)? {
            CyreneLockfile::default()
        } else {
            let lockfile_read =
                fs::read_to_string(&self.lockfile_path).map_err(CyreneError::LockfileRead)?;
            let lockfile: CyreneLockfile =
                toml::de::from_str(&lockfile_read).map_err(CyreneError::LockfileDeserialize)?;
            lockfile
        };
        lockfile.loaded_lockfile = None;
        let lockfile_write =
            toml::ser::to_string(&lockfile).map_err(CyreneError::LockfileSerialize)?;
        fs::write(&self.lockfile_path, lockfile_write).map_err(CyreneError::LockfileWrite)?;
        Ok(())
    }

    pub fn use_local_lockfile(&self, loaded_lockfile: &Path) -> Result<(), CyreneError> {
        let mut lockfile = if !fs::exists(&self.lockfile_path).map_err(CyreneError::LockfileRead)? {
            CyreneLockfile::default()
        } else {
            let lockfile_read =
                fs::read_to_string(&self.lockfile_path).map_err(CyreneError::LockfileRead)?;
            let lockfile: CyreneLockfile =
                toml::de::from_str(&lockfile_read).map_err(CyreneError::LockfileDeserialize)?;
            lockfile
        };
        lockfile.loaded_lockfile = Some(
            fs::canonicalize(loaded_lockfile)
                .map_err(|e| CyreneError::LockfileLocalRead(loaded_lockfile.to_path_buf(), e))?
                .to_string_lossy()
                .to_string(),
        );
        let lockfile_write =
            toml::ser::to_string(&lockfile).map_err(CyreneError::LockfileSerialize)?;
        fs::write(&self.lockfile_path, lockfile_write).map_err(CyreneError::LockfileWrite)?;
        Ok(())
    }

    pub fn load_version_map_from_current_lockfile(
        &self,
    ) -> Result<BTreeMap<String, String>, CyreneError> {
        let mut lockfile = if !fs::exists(&self.lockfile_path).map_err(CyreneError::LockfileRead)? {
            CyreneLockfile::default()
        } else {
            let lockfile_read =
                fs::read_to_string(&self.lockfile_path).map_err(CyreneError::LockfileRead)?;
            let lockfile: CyreneLockfile =
                toml::de::from_str(&lockfile_read).map_err(CyreneError::LockfileDeserialize)?;
            lockfile
        };
        if let Some(loaded_lockfile) = lockfile.loaded_lockfile {
            // Load needed versions from new lockfile
            lockfile = {
                let lockfile_read = fs::read_to_string(&loaded_lockfile).map_err(|e| {
                    CyreneError::LockfileLocalRead(PathBuf::from(loaded_lockfile), e)
                })?;
                let lockfile: CyreneLockfile =
                    toml::de::from_str(&lockfile_read).map_err(CyreneError::LockfileDeserialize)?;
                lockfile
            };
        }
        Ok(lockfile.versions.clone())
    }
}
