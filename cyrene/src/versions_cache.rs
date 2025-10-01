use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::errors::CyreneError;

#[derive(Serialize, Deserialize)]
pub struct CyreneVersionsCache {
    pub versions: BTreeMap<String, Vec<String>>,
}

pub struct CyreneVersionCacheManager {
    cache_path: PathBuf,
}
impl CyreneVersionCacheManager {
    pub fn new(cache_path: &Path) -> Self {
        Self {
            cache_path: cache_path.to_path_buf(),
        }
    }

    pub fn get_versions(&self, name: &str) -> Result<Vec<String>, CyreneError> {
        let cache: CyreneVersionsCache = if !fs::exists(&self.cache_path)? {
            let new_cache = CyreneVersionsCache {
                versions: BTreeMap::new(),
            };
            let new_cache_file = toml::ser::to_string(&new_cache)?;
            fs::write(&self.cache_path, new_cache_file)?;

            new_cache
        } else {
            let file = fs::read_to_string(&self.cache_path)?;
            let cache: CyreneVersionsCache = toml::de::from_str(&file)?;

            cache
        };

        match cache.versions.get(name) {
            Some(some) => Ok(some.clone()),
            None => Ok(Vec::new()),
        }
    }
    pub fn update_version_cache(
        &self,
        name: &str,
        versions: Vec<String>,
    ) -> Result<(), CyreneError> {
        let mut cache: CyreneVersionsCache = if !fs::exists(&self.cache_path)? {
            CyreneVersionsCache {
                versions: BTreeMap::new(),
            }
        } else {
            let file = fs::read_to_string(&self.cache_path)?;
            toml::de::from_str(&file)?
        };
        cache.versions.insert(String::from(name), versions);
        let cache_file = toml::ser::to_string(&cache)?;
        fs::write(&self.cache_path, cache_file)?;

        Ok(())
    }
}
