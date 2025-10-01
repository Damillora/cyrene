use std::{
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use log::debug;
use semver::{Version, VersionReq};

use crate::{
    app::CyreneApp,
    errors::CyreneError,
    lockfile::{self, use_default_lockfile, use_local_lockfile},
    responses::CyreneAppItem,
    versions_cache::{self},
};

pub struct CyreneManager {
    apps_dir: PathBuf,
    plugins_dir: PathBuf,
    exe_dir: PathBuf,
    config_dir: PathBuf,
    cache_path: PathBuf,
}

impl CyreneManager {
    fn get_plugin_script(&self, name: &str) -> PathBuf {
        let mut plugin_file = self.plugins_dir.clone();
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
    fn installation_path(&self, name: &str, version: &str) -> PathBuf {
        let mut installation_dir = self.apps_dir.clone();
        installation_dir.push(format!("{}/{}", name, version));

        installation_dir
    }
    fn installation_root(&self, name: &str) -> PathBuf {
        let mut installation_dir = self.apps_dir.clone();
        installation_dir.push(name);

        installation_dir
    }
    fn lockfile_path(&self) -> PathBuf {
        let mut lockfile_path = self.config_dir.clone();
        lockfile_path.push("cyrene.toml");

        lockfile_path
    }
}

impl CyreneManager {
    pub fn new(proj_dirs: ProjectDirs) -> Result<Self, CyreneError> {
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
        let exe_dir = std::env::current_exe()?
            .parent()
            .ok_or(CyreneError::ExePathError)?
            .to_path_buf();
        let mut versions_cache_dir = proj_dirs.cache_dir().to_path_buf();
        versions_cache_dir.push("versions.yaml");
        fs::create_dir_all(&apps_dir)?;
        fs::create_dir_all(&plugins_dir)?;
        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(proj_dirs.cache_dir())?;
        Ok(Self {
            apps_dir,
            plugins_dir,
            config_dir,
            exe_dir,
            cache_path: versions_cache_dir,
        })
    }
    pub fn list(&self) -> Result<Vec<CyreneAppItem>, CyreneError> {
        let apps = self.get_all_apps()?;
        Ok(apps)
    }

    pub fn versions(&self, name: &str) -> Result<Vec<String>, CyreneError> {
        let cache_path = &self.cache_path;
        let versions = versions_cache::get_versions(cache_path, name)?;
        if versions.is_empty() {
            self.update_versions(name)?;
            return versions_cache::get_versions(cache_path, name);
        }
        Ok(versions)
    }
    pub fn update_versions(&self, name: &str) -> Result<(), CyreneError> {
        let plugin_path = self.get_plugin_script(name);
        let cache_path = &self.cache_path;
        let mut cyrene_app = CyreneApp::new(&plugin_path)?;
        let versions = cyrene_app
            .get_versions()?
            .iter()
            .map(|f| f.to_string())
            .collect();
        versions_cache::update_version_cache(cache_path, name, versions)?;
        Ok(())
    }
    pub fn install(&self, name: &str, version: &str) -> Result<(), CyreneError> {
        self.install_specific_version(name, version)?;
        self.link_binaries(name, version, false, true)?;

        Ok(())
    }

    fn install_specific_version(
        &self,
        name: &str,
        required_version: &str,
    ) -> Result<(), CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let mut cyrene_app = CyreneApp::new(&plugin_path)?;

        println!(
            "Installing {} version {}",
            cyrene_app.app_name(),
            required_version
        );
        // $CYRENE_APPS_DIR/app_name-app_version
        let installation_path = self.installation_path(
            cyrene_app.app_name().as_str(),
            required_version.to_string().as_str(),
        );
        fs::create_dir_all(&installation_path)?;

        cyrene_app.install(&installation_path, required_version.to_string().as_str())?;

        // Update cyrene.toml

        Ok(())
    }

    pub fn link_binaries(
        &self,
        name: &str,
        version: &str,
        overwrite: bool,
        update_lockfile: bool,
    ) -> Result<(), CyreneError> {
        let plugin_path = self.get_plugin_script(name);
        let mut cyrene_app = CyreneApp::new(&plugin_path)?;
        println!("Linking {} version {}", cyrene_app.app_name(), version);

        let installation_path = self.installation_path(cyrene_app.app_name().as_str(), version);
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::AppVersionNotInstalledError);
        }
        let binaries = cyrene_app.binaries(version)?;
        let mut not_overwritten_exists = false;
        for (bin_name, bin_path) in binaries {
            let mut canonical_path = installation_path.clone();
            canonical_path.push(&bin_path);
            let mut exe_path = self.exe_dir.clone();
            exe_path.push(&bin_name);

            if !fs::exists(&exe_path)? {
                debug!(
                    "linking {} to {}",
                    exe_path.to_string_lossy(),
                    canonical_path.to_string_lossy()
                );
                symlink::symlink_file(canonical_path, exe_path)?;
            } else {
                let symlink_path = fs::read_link(&exe_path)?;
                if overwrite {
                    debug!(
                        "overwriting {}  from {} to {}",
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
            }
        }
        if not_overwritten_exists && !overwrite {
            println!(
                "An existing version is already installed. To use the newly installed binaries, run:"
            );
            println!();
            println!("    cyrene link {} {}", cyrene_app.app_name(), version);
        } else if update_lockfile {
            let lockfile = self.lockfile_path();
            lockfile::update_lockfile(&lockfile, name, version)?;
        }

        Ok(())
    }

    pub fn unlink_binaries(&self, name: &str) -> Result<(), CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let lockfile = self.lockfile_path();
        let version = lockfile::find_installed_version_from_lockfile(&lockfile, name)?
            .ok_or(CyreneError::AppVersionNotInLockfileError)?;

        let mut cyrene_app = CyreneApp::new(&plugin_path)?;
        let installation_path =
            self.installation_path(cyrene_app.app_name().as_str(), version.as_str());
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::AppVersionNotInstalledError);
        }
        let binaries = cyrene_app.binaries(version.as_str())?;
        for (bin_name, bin_path) in binaries {
            let mut canonical_path = installation_path.clone();
            canonical_path.push(&bin_path);
            let mut exe_path = self.exe_dir.clone();
            exe_path.push(&bin_name);

            if fs::exists(&exe_path)? {
                debug!("unlinking {}", exe_path.to_string_lossy(),);
                fs::remove_file(&exe_path)?;
            }
        }
        Ok(())
    }

    pub fn package_exists(&self, name: &str, version: &str) -> Result<bool, CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let cyrene_app = CyreneApp::new(&plugin_path)?;
        let installation_path = self.installation_path(cyrene_app.app_name().as_str(), version);
        Ok(fs::exists(&installation_path)?)
    }

    pub fn find_installed_version(&self, name: &str) -> Result<Option<String>, CyreneError> {
        let lockfile = self.lockfile_path();
        let current_version = lockfile::find_installed_version_from_lockfile(&lockfile, name)?;

        Ok(current_version)
    }

    pub fn find_installed_major_release(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Option<String>, CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let cyrene_app = CyreneApp::new(&plugin_path)?;
        let installation_root = self.installation_root(cyrene_app.app_name().as_str());
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
        let plugin_path = self.get_plugin_script(name);

        let cyrene_app = CyreneApp::new(&plugin_path)?;
        let installation_path = self.installation_root(cyrene_app.app_name().as_str());

        Ok(fs::exists(&installation_path)?)
    }

    pub fn uninstall(&self, name: &str, version: &str) -> Result<(), CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let cyrene_app = CyreneApp::new(&plugin_path)?;
        let installation_path = self.installation_path(cyrene_app.app_name().as_str(), version);
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::AppVersionNotInstalledError);
        }

        let lockfile = self.lockfile_path();
        let current_version = lockfile::find_installed_version_from_lockfile(&lockfile, name)?
            .ok_or(CyreneError::AppVersionNotInLockfileError)?;
        let uninstalled_is_linked_version = current_version.eq(version);
        if uninstalled_is_linked_version {
            self.unlink_binaries(name)?;
        }

        fs::remove_dir_all(&installation_path)?;

        if uninstalled_is_linked_version {
            let get_release = self.find_installed_major_release(name, "*")?;
            if let Some(get_release) = get_release {
                self.link_binaries(name, &get_release, true, true)?;
            }
        }

        println!("Uninstalled {} version {}", name, version);
        Ok(())
    }

    pub fn uninstall_all(&self, name: &str) -> Result<(), CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let cyrene_app = CyreneApp::new(&plugin_path)?;
        let installation_path = self.installation_root(cyrene_app.app_name().as_str());
        debug!("{}", installation_path.to_string_lossy());
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::AppVersionNotInstalledError);
        }
        println!("Uninstalling {}", &name);

        self.unlink_binaries(name)?;
        fs::remove_dir_all(&installation_path)?;

        println!("Uninstalled {}", name);
        Ok(())
    }

    pub fn upgrade(
        &self,
        name: &str,
        old_version: &str,
        new_version: &str,
    ) -> Result<(), CyreneError> {
        let current_installed = self.get_current_version(name)?;
        let overwrite_installed = current_installed.eq(old_version);
        self.install_specific_version(name, new_version)?;
        self.link_binaries(name, new_version, overwrite_installed, true)?;
        self.uninstall(name, old_version)?;

        Ok(())
    }

    pub fn get_current_version(&self, name: &str) -> Result<String, CyreneError> {
        let lockfile = self.lockfile_path();
        let old_version = lockfile::find_installed_version_from_lockfile(&lockfile, name)?
            .ok_or(CyreneError::AppVersionNotInLockfileError)?;

        Ok(old_version)
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
            .ok_or(CyreneError::AppVersionNotFoundError)?
            .to_string())
    }

    pub fn verify_version_exists(&self, name: &str, version: &str) -> Result<bool, CyreneError> {
        let versions = self.versions(name)?;

        Ok(versions.iter().any(|f| f.eq(&version)))
    }

    pub fn get_all_apps(&self) -> Result<Vec<CyreneAppItem>, CyreneError> {
        let installation_root = self.apps_dir.clone();
        let list_dirs = fs::read_dir(installation_root)?;
        let apps: Vec<_> = list_dirs
            .filter_map(|p| p.ok())
            .map(|f| {
                let installation_root = f.path().clone();
                let app_name = f.path().file_name().unwrap().to_string_lossy().to_string();
                let list_dirs: Vec<_> = fs::read_dir(installation_root)?.collect();

                let mut a: Vec<String> = list_dirs
                    .iter()
                    .filter_map(|p| p.as_ref().ok())
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
                        name: app_name.clone(),
                        version: f.to_string(),
                    })
                    .collect();
                Ok(a)
            })
            .filter_map(|p: Result<Vec<CyreneAppItem>, CyreneError>| p.ok())
            .flatten()
            .collect();
        Ok(apps)
    }

    pub fn load_lockfile(&self, loaded_lockfile: Option<&Path>) -> Result<(), CyreneError> {
        let default_lockfile = self.lockfile_path();
        match &loaded_lockfile {
            Some(loaded_lockfile) => use_local_lockfile(&default_lockfile, loaded_lockfile)?,
            None => use_default_lockfile(&default_lockfile)?,
        };
        let lockfile_items = lockfile::load_versions_from_lockfile(&default_lockfile)?;
        if lockfile_items
            .iter()
            .any(|h| match self.verify_version_exists(&h.name, &h.version) {
                Ok(t) => !t,
                Err(_) => true,
            })
        {
            return Err(CyreneError::LockfileAppVersionError);
        }
        for lockfile_item in lockfile_items {
            if !self.package_exists(&lockfile_item.name, &lockfile_item.version)? {
                self.install_specific_version(&lockfile_item.name, &lockfile_item.version)?;
            }
            self.link_binaries(&lockfile_item.name, &lockfile_item.version, true, false)?;
        }

        Ok(())
    }
}
