use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{errors::CyreneError, responses::CyreneAppItem};

#[derive(Serialize, Deserialize)]
pub struct CyreneLockfile {
    pub versions: BTreeMap<String, String>,
}
pub fn load_lockfile(lockfile_path: &Path) -> Result<Vec<CyreneAppItem>, CyreneError> {
    if !fs::exists(&lockfile_path)? {
        Err(CyreneError::LockfileNotFoundError)
    } else {
        Ok(Vec::new())
    }
}
pub fn get_current_version_from_lockfile(
    lockfile_path: &Path,
    name: &str,
) -> Result<Option<String>, CyreneError> {
    let lockfile = if !fs::exists(&lockfile_path)? {
        CyreneLockfile {
            versions: BTreeMap::new(),
        }
    } else {
        let lockfile_read = fs::read_to_string(&lockfile_path)?;
        let lockfile: CyreneLockfile = toml::de::from_str(&lockfile_read)?;
        lockfile
    };
    let version = lockfile.versions.get(name).map(|x| x.to_string());
    Ok(version)
}

pub fn update_lockfile(lockfile_path: &Path, name: &str, version: &str) -> Result<(), CyreneError> {
    let mut lockfile = if !fs::exists(&lockfile_path)? {
        CyreneLockfile {
            versions: BTreeMap::new(),
        }
    } else {
        let lockfile_read = fs::read_to_string(&lockfile_path)?;
        let lockfile: CyreneLockfile = toml::de::from_str(&lockfile_read)?;
        lockfile
    };
    lockfile
        .versions
        .insert(name.to_owned(), version.to_owned());
    let lockfile_write = toml::ser::to_string(&lockfile)?;
    fs::write(lockfile_path, lockfile_write)?;
    Ok(())
}
