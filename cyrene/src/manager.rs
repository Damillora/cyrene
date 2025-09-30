use std::{
    cmp::{self, Ordering},
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use log::debug;
use versions::{Requirement, Version, Versioning};

use crate::{app::CyreneApp, errors::CyreneError, lockfile, responses::CyreneAppVersions, tables};

pub struct CyreneManager {
    apps_dir: PathBuf,
    plugins_dir: PathBuf,
    exe_dir: PathBuf,
    config_dir: PathBuf,
}

impl CyreneManager {
    fn get_plugin_script(&self, name: &str) -> PathBuf {
        let mut plugin_file = self.plugins_dir.clone();
        plugin_file.push(format!("{}.rn", name));

        plugin_file
    }
    fn search_in_version(
        &self,
        versions: Vec<CyreneAppVersions>,
        version_range: &str,
    ) -> Option<Versioning> {
        if let Some(ver) = Versioning::new(version_range) {
            if versions.iter().any(|f| f.version.eq(&ver)) {
                return Some(ver);
            }
        }

        if let Some(requirement) = Requirement::new(version_range) {
            if let Some(ver) = versions.iter().find(|f| requirement.matches(&f.version)) {
                return Some(ver.version.clone());
            }
        }

        return None;
    }
    fn installation_path(&self, name: &str, version: &str) -> PathBuf {
        let mut installation_dir = self.apps_dir.clone();
        installation_dir.push(format!("{}/{}", name, version));

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
        fs::create_dir_all(&apps_dir)?;
        fs::create_dir_all(&plugins_dir)?;
        fs::create_dir_all(&config_dir)?;
        Ok(Self {
            apps_dir,
            plugins_dir,
            config_dir,
            exe_dir,
        })
    }
    pub fn list(&self) -> Result<(), CyreneError> {
        Ok(())
    }

    pub fn versions(&self, name: &str) -> Result<Vec<CyreneAppVersions>, CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let mut cyrene_app = CyreneApp::new(&plugin_path)?;
        let versions = cyrene_app.get_versions()?;
        Ok(versions)
    }
    pub fn install(&self, name: &str, version: Option<&str>) -> Result<(), CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let mut cyrene_app = CyreneApp::new(&plugin_path)?;
        let versions = cyrene_app.get_versions()?;

        // Get needed version
        let required_version = match version {
            // Find the latest suitable version for given version
            Some(version_range) => self
                .search_in_version(versions, version_range)
                .ok_or(CyreneError::NoAppError)?,
            // Use latest version otherwise
            None => versions.get(0).unwrap().version.clone(),
        };
        self.install_specific_version(name, &required_version.to_string())?;
        self.link_binaries(name, required_version.to_string().as_str(), false)?;

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
    ) -> Result<(), CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let mut cyrene_app = CyreneApp::new(&plugin_path)?;
        let installation_path = self.installation_path(cyrene_app.app_name().as_str(), version);
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::NoAppError);
        }
        let lockfile = self.lockfile_path();
        lockfile::update_lockfile(&lockfile, name, version)?;
        let binaries = cyrene_app.binaries(version)?;
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
                    println!("Linked version {} of {}", version, cyrene_app.app_name());
                } else {
                    debug!(
                        "{} is already pointing to {}",
                        exe_path.to_string_lossy(),
                        symlink_path.to_string_lossy()
                    );
                    println!(
                        "An existing version is already installed. To use the newly installed binaries, run:"
                    );
                    println!();
                    println!("    cyrene link {} {}", cyrene_app.app_name(), version);
                }
            }
        }
        Ok(())
    }

    pub fn unlink_binaries(&self, name: &str) -> Result<(), CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let lockfile = self.lockfile_path();
        let version = lockfile::get_current_version_from_lockfile(&lockfile, &name)?
            .ok_or(CyreneError::NoAppError)?;

        let mut cyrene_app = CyreneApp::new(&plugin_path)?;
        let installation_path =
            self.installation_path(cyrene_app.app_name().as_str(), version.as_str());
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::NoAppError);
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
        return Ok(fs::exists(&installation_path)?);
    }

    pub fn uninstall(&self, name: &str, version: &str) -> Result<(), CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let cyrene_app = CyreneApp::new(&plugin_path)?;
        let installation_path = self.installation_path(cyrene_app.app_name().as_str(), version);
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::NoAppError);
        }

        fs::remove_dir_all(&installation_path)?;

        println!("Uninstalled {} version {}", name, version);
        Ok(())
    }

    pub fn uninstall_all(&self, name: &str) -> Result<(), CyreneError> {
        let plugin_path = self.get_plugin_script(name);

        let cyrene_app = CyreneApp::new(&plugin_path)?;
        let installation_path = self.installation_path(cyrene_app.app_name().as_str(), "");
        if !fs::exists(&installation_path)? {
            return Err(CyreneError::NoAppError);
        }

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
        self.install_specific_version(name, new_version)?;
        self.link_binaries(name, new_version, true)?;
        self.uninstall(name, old_version)?;

        Ok(())
    }

    pub fn get_current_version(&self, name: &str) -> Result<String, CyreneError> {
        let lockfile = self.lockfile_path();
        let old_version = lockfile::get_current_version_from_lockfile(&lockfile, &name)?
            .ok_or(CyreneError::NoAppError)?;

        Ok(old_version)
    }

    pub fn get_latest_version_major(
        &self,
        name: &str,
        old_version: &str,
    ) -> Result<String, CyreneError> {
        let plugin_path = self.get_plugin_script(name);
        let mut cyrene_app = CyreneApp::new(&plugin_path)?;

        let versions = cyrene_app.get_versions()?;

        // Get needed version
        let required_version = self
            .search_in_version(versions, format!("^{}", old_version).as_str())
            .ok_or(CyreneError::NoAppError)?;

        Ok(required_version.to_string())
    }
}
