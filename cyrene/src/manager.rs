use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use log::debug;
use semver::Version;

use crate::{
    app::CyreneApp, dirs::CyreneDirs, errors::CyreneError, lockfile::CyreneLockfileManager,
    transaction::TransactionCommands, util, versions_cache::CyreneVersionCacheManager,
};

pub struct CyreneManager {
    dirs: Arc<CyreneDirs>,
    lockfile: Box<CyreneLockfileManager>,
    version_cache: Box<CyreneVersionCacheManager>,
}

// Private functions
impl CyreneManager {
    fn get_app_path(&self, name: &str) -> PathBuf {
        let mut app_path = self.dirs.plugins_dir.clone();
        app_path.push(format!("{}.cyrene", name));

        app_path
    }
    fn verify_version_exists(&self, name: &str, version: &str) -> Result<bool, CyreneError> {
        let versions = self.version_cache.get_versions(name)?;

        Ok(versions.iter().any(|f| f.eq(&version)))
    }
}
impl CyreneManager {
    pub fn new(
        dirs: Arc<CyreneDirs>,
        lockfile_manager: Box<CyreneLockfileManager>,
        cache_manager: Box<CyreneVersionCacheManager>,
    ) -> Self {
        Self {
            dirs,
            lockfile: lockfile_manager,
            version_cache: cache_manager,
        }
    }

    pub fn load_app(&self, name: &str) -> Result<CyreneApp, CyreneError> {
        let plugin_path = self.get_app_path(name);
        CyreneApp::from_file(&plugin_path)
    }

    pub fn list_apps(&self) -> Result<Vec<String>, CyreneError> {
        let installation_root = self.dirs.apps_dir.clone();
        let list_dirs = fs::read_dir(&installation_root)
            .map_err(|e| CyreneError::AppList(installation_root.to_path_buf(), e))?;
        let apps: Vec<_> = list_dirs
            .filter_map(|p| p.ok())
            .map(|f| f.path().file_name().unwrap().to_string_lossy().to_string())
            .collect();

        Ok(apps)
    }

    pub fn get_app_version_map(&self) -> Result<BTreeMap<String, String>, CyreneError> {
        self.lockfile.load_version_map_from_current_lockfile()
    }

    pub fn list_installed_app_versions(
        &self,
        name: &str,
    ) -> Result<Vec<(String, String)>, CyreneError> {
        let installation_root = self.dirs.installation_root(name);
        let list_dirs = fs::read_dir(installation_root)
            .map_err(|e| CyreneError::AppCheck(name.to_string(), "".to_string(), e))?;

        let mut a: Vec<String> = list_dirs
            .filter_map(|p| p.ok())
            .map(|p| p.path().file_name().unwrap().to_string_lossy().to_string())
            .collect();
        a.sort_by(|a, b| {
            let a = Version::parse(a).unwrap();
            let b = Version::parse(b).unwrap();
            b.cmp(&a)
        });
        let a = a
            .iter()
            .map(|f| (name.to_string(), f.to_string()))
            .collect();
        Ok(a)
    }

    pub async fn versions(&self, name: &str) -> Result<Vec<String>, CyreneError> {
        let versions = self.version_cache.get_versions(name)?;
        if versions.is_empty() {
            self.update_versions(name).await?;
            return self.version_cache.get_versions(name);
        }
        Ok(versions)
    }

    pub async fn update_versions(&self, name: &str) -> Result<(), CyreneError> {
        let app = self.load_app(name)?;
        let versions = app.get_versions().await?;
        self.version_cache.update_version_cache(name, versions)?;
        Ok(())
    }

    pub fn find_installed_version(&self, name: &str) -> Result<Option<String>, CyreneError> {
        self.lockfile.find_installed_version_from_lockfile(name)
    }

    pub fn find_installed_major_release(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Option<String>, CyreneError> {
        let installation_root = self.dirs.installation_root(name);
        if !fs::exists(&installation_root)
            .map_err(|e| CyreneError::AppCheck(name.to_string(), version.to_string(), e))?
        {
            return Ok(None);
        }
        let list_dirs = fs::read_dir(installation_root)
            .map_err(|e| CyreneError::AppCheck(name.to_string(), version.to_string(), e))?;

        let mut a: Vec<String> = list_dirs
            .filter_map(|p| p.ok())
            .map(|p| p.path().file_name().unwrap().to_string_lossy().to_string())
            .collect();
        a.sort_by(|a, b| {
            let a = Version::parse(a).unwrap();
            let b = Version::parse(b).unwrap();
            b.cmp(&a)
        });
        let latest_installed_release = util::search_in_version(a, version);

        let a = latest_installed_release;
        Ok(a)
    }

    pub async fn get_latest_version(&self, name: &str) -> Result<String, CyreneError> {
        let versions = self.versions(name).await?;

        Ok(versions
            .first()
            .ok_or(CyreneError::AppVersionNotCached(name.to_string()))?
            .to_string())
    }

    pub async fn get_latest_major_release(
        &self,
        name: &str,
        old_version: &str,
    ) -> Result<Option<String>, CyreneError> {
        let versions = self.versions(name).await?;

        // Get needed version
        let required_version = util::search_in_version(versions, old_version);

        Ok(required_version)
    }

    pub fn is_version_installed(&self, name: &str, version: &str) -> Result<bool, CyreneError> {
        let installation_path = self.dirs.installation_path(name, version);
        fs::exists(&installation_path)
            .map_err(|e| CyreneError::AppCheck(name.to_string(), version.to_string(), e))
    }

    pub fn check_upgrade_latest(&self, name: &str) -> Result<bool, CyreneError> {
        let app = self.load_app(name)?;
        let upgrade_latest = app.upgrade_latest();

        Ok(upgrade_latest)
    }

    pub async fn load_lockfile(
        &self,
        loaded_lockfile: Option<&Path>,
    ) -> Result<Vec<TransactionCommands>, CyreneError> {
        match &loaded_lockfile {
            Some(loaded_lockfile) => self.lockfile.use_local_lockfile(loaded_lockfile)?,
            None => self.lockfile.use_default_lockfile()?,
        };
        let lockfile_items = self.lockfile.load_version_map_from_current_lockfile()?;
        if let Some(nonexistent_app) =
            lockfile_items
                .iter()
                .find(|h| match self.verify_version_exists(h.0, h.1) {
                    Ok(t) => !t,
                    Err(_) => true,
                })
        {
            return Err(CyreneError::LockfileAppVersion(
                nonexistent_app.0.to_string(),
                nonexistent_app.1.to_string(),
            ));
        }
        let mut transactions = Vec::new();
        for lockfile_item in lockfile_items {
            if !self.is_version_installed(&lockfile_item.0, &lockfile_item.1)? {
                transactions.push(TransactionCommands::Install {
                    app: lockfile_item.0.clone(),
                    version: lockfile_item.1.clone(),
                });
            }
            transactions.push(TransactionCommands::Link {
                app: lockfile_item.0.clone(),
                version: lockfile_item.1.clone(),
                overwrite: true,
            });
        }

        Ok(transactions)
    }
    // Transactions
    pub async fn install_version(&self, name: &str, version: &str) -> Result<(), CyreneError> {
        let installation_path = self.dirs.ensure_installation_dir(name, version)?;
        let app = self.load_app(name)?;
        app.install(version, &installation_path).await?;

        Ok(())
    }
    // Transactions
    pub async fn post_install_version(&self, name: &str, version: &str) -> Result<(), CyreneError> {
        let installation_path = self.dirs.installation_path(name, version);
        let app = self.load_app(name)?;
        app.post_install(version, &installation_path).await?;

        Ok(())
    }
    pub fn update_lockfile(&self, name: &str, version: Option<&str>) -> Result<(), CyreneError> {
        debug!(
            "Updating lockfile: app version {:?} for plugin {}",
            version, &name
        );
        self.lockfile.update_lockfile(name, version)
    }
    pub fn link_binaries(
        &self,
        name: &str,
        version: &str,
        overwrite: bool,
    ) -> Result<bool, CyreneError> {
        let app = self.load_app(name)?;
        debug!("Using app version {} for plugin {}", version, &name);

        let is_installed = self.is_version_installed(name, version)?;
        if !is_installed {
            return Err(CyreneError::AppNotInstalled(
                name.to_string(),
                version.to_string(),
            ));
        }
        let installation_path = self.dirs.installation_path(name, version);
        debug!(
            "Using installation dir {}",
            installation_path.to_string_lossy()
        );

        let binaries = app.binaries(version)?;
        let mut not_overwritten_exists = false;

        for (bin_name, bin_path) in binaries {
            let mut canonical_path = installation_path.clone();
            canonical_path.push(&bin_path);
            let mut exe_path = self.dirs.exe_dir.clone();
            exe_path.push(&bin_name);
            debug!(
                "Attempting to link {} to {}",
                canonical_path.to_string_lossy(),
                exe_path.to_string_lossy()
            );

            // Sanity check
            let current_exe = std::env::current_exe().map_err(CyreneError::ExeCheck)?;
            if current_exe.eq(&exe_path) {
                // Stop Cyrene from sacrificing herself to the Remembrance
                return Err(CyreneError::AppLinkingToSelf);
            }

            if let Ok(metadata) = fs::metadata(&exe_path) {
                let symlink_path = if metadata.is_symlink() {
                    fs::read_link(&exe_path).map_err(|e| {
                        CyreneError::AppLinkRead(exe_path.to_string_lossy().to_string(), e)
                    })?
                } else {
                    exe_path.clone()
                };
                if overwrite {
                    debug!(
                        "overwriting {} from {} to {}",
                        exe_path.to_string_lossy(),
                        symlink_path.to_string_lossy(),
                        canonical_path.to_string_lossy()
                    );
                    fs::remove_file(&exe_path).map_err(|e| {
                        CyreneError::AppLinkRemove(exe_path.to_string_lossy().to_string(), e)
                    })?;
                    symlink::symlink_file(&canonical_path, &exe_path).map_err(|e| {
                        CyreneError::AppLinkCreate(
                            exe_path.to_string_lossy().to_string(),
                            canonical_path.to_string_lossy().to_string(),
                            e,
                        )
                    })?;
                } else {
                    not_overwritten_exists = true;
                    debug!(
                        "{} is already pointing to {}",
                        exe_path.to_string_lossy(),
                        symlink_path.to_string_lossy()
                    );
                }
            } else {
                debug!(
                    "linking {} to {}",
                    exe_path.to_string_lossy(),
                    canonical_path.to_string_lossy()
                );
                symlink::symlink_file(&canonical_path, &exe_path).map_err(|e| {
                    CyreneError::AppLinkCreate(
                        exe_path.to_string_lossy().to_string(),
                        canonical_path.to_string_lossy().to_string(),
                        e,
                    )
                })?;
            }
        }

        Ok(not_overwritten_exists)
    }

    pub fn unlink_binaries(&self, name: &str) -> Result<(), CyreneError> {
        let app = self.load_app(name)?;
        debug!("Unlinking app versions for plugin {}", &name);

        let binaries = app.binaries("")?;

        for (bin_name, _) in binaries {
            let mut exe_path = self.dirs.exe_dir.clone();
            exe_path.push(&bin_name);

            if fs::exists(&exe_path)
                .map_err(|e| CyreneError::AppLinkRead(exe_path.to_string_lossy().to_string(), e))?
            {
                debug!("unlinking {}", exe_path.to_string_lossy(),);
                fs::remove_file(&exe_path).map_err(|e| {
                    CyreneError::AppLinkRemove(exe_path.to_string_lossy().to_string(), e)
                })?;
            }
        }
        Ok(())
    }
    pub fn uninstall_version(&self, name: &str, version: &str) -> Result<(), CyreneError> {
        debug!("Uninstalling app version {} for plugin {}", version, name);

        let is_installed = self.is_version_installed(name, version)?;
        if !is_installed {
            return Err(CyreneError::AppNotInstalled(
                name.to_string(),
                version.to_string(),
            ));
        }
        let installation_path = self.dirs.installation_path(name, version);

        fs::remove_dir_all(&installation_path)
            .map_err(|e| CyreneError::AppRemove(name.to_string(), version.to_string(), e))?;

        Ok(())
    }

    pub fn uninstall_all(&self, name: &str) -> Result<(), CyreneError> {
        debug!("Uninstalling app versions for plugin {}", name);
        let installation_path = self.dirs.installation_root(name);
        if !fs::exists(&installation_path)
            .map_err(|e| CyreneError::AppCheck(name.to_string(), "".to_string(), e))?
        {
            return Err(CyreneError::AppNotInstalled(
                name.to_string(),
                "".to_string(),
            ));
        }
        fs::remove_dir_all(&installation_path)
            .map_err(|e| CyreneError::AppRemove(name.to_string(), "".to_string(), e))?;

        Ok(())
    }
}
