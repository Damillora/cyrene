use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use console::style;
use log::debug;
use semver::{Version, VersionReq};

use crate::{
    app::CyreneApp, dirs::CyreneDirs, errors::CyreneError, lockfile::CyreneLockfileManager,
    responses::CyreneAppItem, versions_cache::CyreneVersionCacheManager,
};

pub struct CyreneManager {
    dirs: Arc<CyreneDirs>,
    lockfile: Box<CyreneLockfileManager>,
    version_cache: Box<CyreneVersionCacheManager>,
}

impl CyreneManager {
    fn get_plugin_script(&self, name: &str) -> PathBuf {
        let mut plugin_file = self.dirs.plugins_dir.clone();
        plugin_file.push(format!("{}.rn", name));

        plugin_file
    }
    fn search_in_version(&self, versions: Vec<String>, version_range: &str) -> Option<String> {
        let versionings: Vec<Version> = versions
            .iter()
            .map(|f| Version::parse(f))
            .filter_map(|f| f.ok())
            .collect();

        if let Ok(requirement) = VersionReq::parse(version_range)
            && let Some(ver) = versionings.iter().find(|f| requirement.matches(f))
        {
            return Some(ver.to_string());
        }

        None
    }
    /// Link all binaries for a specific version installed by this plugin
    /// Returns a bool whether binaries are actually linked, or if there are existing links
    fn link_plugin_binaries(
        &self,
        plugin: &mut CyreneApp,
        version: &str,
        overwrite: bool,
    ) -> Result<bool, CyreneError> {
        let plugin_name = plugin.plugin_name();
        debug!("Using app version {} for plugin {}", version, &plugin_name);

        let installation_path = self.dirs.installation_path(&plugin_name, version);
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::AppVersionNotInstalledError(
                version.to_string(),
                plugin_name,
            ));
        }
        debug!(
            "Using installation dir {}",
            installation_path.to_string_lossy()
        );

        let binaries = plugin.binaries(version)?;
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
            let current_exe = std::env::current_exe()?;
            if current_exe.eq(&exe_path) {
                // Stop Cyrene from sacrificing herself to the Remembrance
                return Err(CyreneError::AppLinkingToItselfError);
            }

            if let Ok(metadata) = fs::metadata(&exe_path) {
                let symlink_path = if metadata.is_symlink() {
                    fs::read_link(&exe_path)?
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
                    fs::remove_file(&exe_path)?;
                    symlink::symlink_file(canonical_path, &exe_path)?;
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
                symlink::symlink_file(canonical_path, exe_path)?;
            }
        }

        Ok(not_overwritten_exists)
    }

    fn unlink_plugin_binaries(
        &mut self,
        plugin: &mut CyreneApp,
        version: &str,
    ) -> Result<(), CyreneError> {
        let plugin_name = plugin.plugin_name();
        debug!("Unlinking app versions for plugin {}", &plugin_name);
        let installation_path = self.dirs.installation_path(&plugin_name, version);
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::AppVersionNotInstalledError(
                version.to_string(),
                plugin_name,
            ));
        }
        let binaries = plugin.binaries(version)?;

        for (bin_name, bin_path) in binaries {
            let mut canonical_path = installation_path.clone();
            canonical_path.push(&bin_path);
            let mut exe_path = self.dirs.exe_dir.clone();
            exe_path.push(&bin_name);

            if fs::exists(&exe_path)? {
                debug!("unlinking {}", exe_path.to_string_lossy(),);
                fs::remove_file(&exe_path)?;
            }
        }
        Ok(())
    }
}

impl CyreneManager {
    pub fn new(
        dirs: Arc<CyreneDirs>,
        lockfile_manager: Box<CyreneLockfileManager>,
        cache_manager: Box<CyreneVersionCacheManager>,
    ) -> Result<Self, CyreneError> {
        Ok(Self {
            dirs,
            lockfile: lockfile_manager,
            version_cache: cache_manager,
        })
    }
    fn load_plugin(&self, name: &str) -> Result<Box<CyreneApp>, CyreneError> {
        let plugin_path = self.get_plugin_script(name);
        CyreneApp::new(&plugin_path)
    }

    pub fn versions(&self, name: &str) -> Result<Vec<String>, CyreneError> {
        let versions = self.version_cache.get_versions(name)?;
        if versions.is_empty() {
            self.update_versions(name)?;
            return self.version_cache.get_versions(name);
        }
        Ok(versions)
    }

    pub fn update_versions(&self, name: &str) -> Result<(), CyreneError> {
        let mut plugin = self.load_plugin(name)?;
        let versions = plugin
            .get_versions()?
            .iter()
            .map(|f| f.to_string())
            .collect();
        self.version_cache.update_version_cache(name, versions)?;
        Ok(())
    }

    pub fn install(&self, name: &str, version: &str) -> Result<(), CyreneError> {
        self.install_specific_version(name, version)?;
        let exists_not_overwritten = self.link_binaries(name, version, false)?;
        if !exists_not_overwritten {
            self.update_lockfile(name, Some(version))?;
        }
        Ok(())
    }

    pub fn install_specific_version(
        &self,
        name: &str,
        required_version: &str,
    ) -> Result<(), CyreneError> {
        let mut plugin = self.load_plugin(name)?;
        debug!(
            "Installing app version {} for plugin {}",
            required_version, &name
        );

        // $CYRENE_APPS_DIR/app_name-app_version
        let installation_path = self
            .dirs
            .installation_path(name, required_version.to_string().as_str());
        fs::create_dir_all(&installation_path)?;

        plugin.install_version(&installation_path, required_version.to_string().as_str())?;

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
        let mut plugin = self.load_plugin(name)?;
        let not_overwritten_exists = self.link_plugin_binaries(&mut plugin, version, overwrite)?;

        Ok(not_overwritten_exists)
    }

    pub fn unlink_binaries(&mut self, name: &str) -> Result<(), CyreneError> {
        let version = self
            .lockfile
            .find_installed_version_from_lockfile(name)?
            .ok_or(CyreneError::AppNotInLockfileError(name.to_string()))?;

        let mut plugin = self.load_plugin(name)?;
        self.unlink_plugin_binaries(&mut plugin, &version)
    }

    pub fn package_exists(&self, name: &str, version: &str) -> Result<bool, CyreneError> {
        let installation_path = self.dirs.installation_path(name, version);
        Ok(fs::exists(&installation_path)?)
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
        if !fs::exists(&installation_root)? {
            return Ok(None);
        }
        let list_dirs = fs::read_dir(installation_root)?;

        let mut a: Vec<String> = list_dirs
            .filter_map(|p| p.ok())
            .map(|p| p.path().file_name().unwrap().to_string_lossy().to_string())
            .collect();
        a.sort_by(|a, b| {
            let a = Version::parse(a).unwrap();
            let b = Version::parse(b).unwrap();
            b.cmp(&a)
        });
        let latest_installed_release = self.search_in_version(a, version);

        let a = latest_installed_release;
        Ok(a)
    }

    pub fn package_root_exists(&self, name: &str) -> Result<bool, CyreneError> {
        let installation_path = self.dirs.installation_root(name);

        Ok(fs::exists(&installation_path)?)
    }

    pub fn uninstall(&mut self, name: &str, version: &str) -> Result<(), CyreneError> {
        debug!("Uninstalling app version {} for plugin {}", version, name);
        let installation_path = self.dirs.installation_path(name, version);
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::AppVersionNotInstalledError(
                version.to_string(),
                name.to_string(),
            ));
        }

        let current_version = self
            .lockfile
            .find_installed_version_from_lockfile(name)?
            .ok_or(CyreneError::AppNotInLockfileError(name.to_string()))?;
        let uninstalled_is_linked_version = current_version.eq(version);
        if uninstalled_is_linked_version {
            debug!(
                "App version {} for plugin {} is in use, unlinking app versions",
                version, name
            );
            self.unlink_binaries(name)?;
        }

        fs::remove_dir_all(&installation_path)?;

        if uninstalled_is_linked_version {
            let get_release = self.find_installed_major_release(name, "*")?;
            if let Some(get_release) = get_release {
                debug!(
                    "Using latest app versions {} for plugin {} after uninstall",
                    get_release, name
                );
                self.link_binaries(name, &get_release, true)?;
                self.update_lockfile(name, Some(&get_release))?;
            } else {
                let installation_root = self.dirs.installation_root(&name);
                fs::remove_dir(installation_root)?;
                self.update_lockfile(name, None)?;
            }
        }

        Ok(())
    }

    pub fn uninstall_all(&mut self, name: &str) -> Result<(), CyreneError> {
        debug!("Uninstalling app versions for plugin {}", name);
        let installation_path = self.dirs.installation_root(name);
        debug!("{}", installation_path.to_string_lossy());
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::AppNotInstalledError(name.to_string()));
        }
        self.unlink_binaries(name)?;
        self.update_lockfile(name, None)?;
        fs::remove_dir_all(&installation_path)?;

        Ok(())
    }

    pub fn upgrade(
        &mut self,
        name: &str,
        old_version: &str,
        new_version: &str,
    ) -> Result<(), CyreneError> {
        debug!(
            "Unpgrading app version {} to {} for plugin {}",
            old_version, new_version, name
        );
        let current_installed = self
            .find_installed_version(name)?
            .ok_or(CyreneError::LockfileAppError(name.to_string()))?;
        let overwrite_installed = current_installed.eq(old_version);
        self.install_specific_version(name, new_version)?;
        self.link_binaries(name, new_version, overwrite_installed)?;
        self.update_lockfile(name, Some(new_version))?;
        self.uninstall(name, old_version)?;

        Ok(())
    }

    pub fn check_upgrade_latest(&self, name: &str) -> Result<bool, CyreneError> {
        let upgrade_latest = self.lockfile.find_upgrade_latest_from_lockfile(name)?;

        Ok(upgrade_latest)
    }
    pub fn get_latest_major_release(
        &self,
        name: &str,
        old_version: &str,
    ) -> Result<Option<String>, CyreneError> {
        let versions = self.versions(name)?;

        // Get needed version
        let required_version = self.search_in_version(versions, old_version);

        Ok(required_version)
    }

    pub fn get_latest_version(&self, name: &str) -> Result<String, CyreneError> {
        let versions = self.versions(name)?;

        Ok(versions
            .first()
            .ok_or(CyreneError::AppVersionNotInCacheError(name.to_string()))?
            .to_string())
    }

    pub fn verify_version_exists(&self, name: &str, version: &str) -> Result<bool, CyreneError> {
        let versions = self.versions(name)?;

        Ok(versions.iter().any(|f| f.eq(&version)))
    }

    pub fn list_apps(&self) -> Result<Vec<String>, CyreneError> {
        let installation_root = self.dirs.apps_dir.clone();
        let list_dirs = fs::read_dir(installation_root)?;
        let apps: Vec<_> = list_dirs
            .filter_map(|p| p.ok())
            .map(|f| f.path().file_name().unwrap().to_string_lossy().to_string())
            .collect();

        Ok(apps)
    }

    pub fn list_installed_app_versions(
        &self,
        name: &str,
    ) -> Result<Vec<CyreneAppItem>, CyreneError> {
        let installation_root = self.dirs.installation_root(name);
        let list_dirs = fs::read_dir(installation_root)?;

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
            .map(|f| CyreneAppItem {
                name: name.to_string(),
                version: f.to_string(),
            })
            .collect();
        Ok(a)
    }
    pub fn list_linked_app_versions(&self) -> Result<Vec<CyreneAppItem>, CyreneError> {
        let lockfile_items: Vec<_> = self
            .lockfile
            .load_versions_from_current_lockfile()?
            .iter()
            .map(|f| CyreneAppItem {
                name: f.name.clone(),
                version: f.version.clone(),
            })
            .collect();
        Ok(lockfile_items)
    }
    pub fn get_app_version_map(&self) -> Result<BTreeMap<String, String>, CyreneError> {
        self.lockfile.load_version_map_from_current_lockfile()
    }
    pub fn load_lockfile(&self, loaded_lockfile: Option<&Path>) -> Result<(), CyreneError> {
        match &loaded_lockfile {
            Some(loaded_lockfile) => self.lockfile.use_local_lockfile(loaded_lockfile)?,
            None => self.lockfile.use_default_lockfile()?,
        };
        let lockfile_items = self.lockfile.load_versions_from_current_lockfile()?;
        if let Some(nonexistent_app) =
            lockfile_items
                .iter()
                .find(|h| match self.verify_version_exists(&h.name, &h.version) {
                    Ok(t) => !t,
                    Err(_) => true,
                })
        {
            return Err(CyreneError::LockfileAppVersionError(
                nonexistent_app.version.to_string(),
                nonexistent_app.name.to_string(),
            ));
        }
        for lockfile_item in lockfile_items {
            if !self.package_exists(&lockfile_item.name, &lockfile_item.version)? {
                self.install_specific_version(&lockfile_item.name, &lockfile_item.version)?;
            }
            self.link_binaries(&lockfile_item.name, &lockfile_item.version, true)?;
        }

        Ok(())
    }
    pub fn generate_env(&self) -> Result<(), CyreneError> {
        // Write a shell script exporting cyrene's CYRENE_INSTALL_DIR
        // Located in $XDG_CONFIG_HOME/cyrene/cyrene.sh
        let mut env_file = self.dirs.config_dir.clone();
        env_file.push("cyrene_env.sh");

        let exists_before = fs::exists(&env_file)?;

        let script = format!(
            "export CYRENE_INSTALL_DIR={}
export CYRENE_APPS_DIR={}
export CYRENE_PLUGINS_DIR={}",
            &self.dirs.exe_dir.to_string_lossy(),
            &self.dirs.apps_dir.to_string_lossy(),
            &self.dirs.plugins_dir.to_string_lossy(),
        );
        fs::write(&env_file, &script)?;

        if !exists_before {
            println!(
                "Import the environment needed to enable managing cyrene with cyrene itself by {}:",
                style("adding this line to your shell profile").fg(console::Color::Yellow)
            );
            println!();
            println!("    source {}", env_file.to_string_lossy());
            println!();
            println!(
                "To start using cyrene now, {}",
                style("copy the line into your shell and run it.").fg(console::Color::Yellow)
            );
        }

        Ok(())
    }
}
