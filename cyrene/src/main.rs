use std::{fs, path::PathBuf};

use clap::{Args, Parser, Subcommand, command};
use inquire::Confirm;
use miette::{ErrReport, IntoDiagnostic};
use semver::Version;

use crate::{
    errors::CyreneError, manager::CyreneManager, tables::CyreneAppVersionsAllRow,
    util::is_major_version_equal,
};
/// Cyrene app definition
pub mod app;
/// Modules used by Cyrene app scripts
pub mod app_module;
/// Directory management
pub mod dirs;
/// Error definitions
pub mod errors;
/// Lockfile
pub mod lockfile;
/// Main Cyrene manager logic
pub mod manager;
/// Cyrene response structs
pub mod responses;
/// Cyrene tables
pub mod tables;
/// Various Cyrene utilities
pub mod util;
/// Cyrene version caching
pub mod versions_cache;

pub struct AppVersion {
    name: String,
    version: Option<String>,
}
pub struct AppVersionAction {
    name: String,
    version: String,
}
pub struct AppVersionUpgradeAction {
    name: String,
    old_version: String,
    new_version: String,
}
impl From<&String> for AppVersion {
    fn from(value: &String) -> Self {
        let app_str: Vec<_> = value.split("@").collect();
        if app_str.len() == 1 {
            AppVersion {
                name: app_str.first().unwrap().to_string(),
                version: None,
            }
        } else {
            AppVersion {
                name: app_str.first().unwrap().to_string(),
                version: Some(app_str.get(1).unwrap().to_string()),
            }
        }
    }
}

/// Manage your installed binaries
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install binaries
    Install(AppInstallOpts),
    /// Upgrade binaries
    Upgrade(AppUpgradeOpts),
    /// Uninstall binaries
    Uninstall(AppUninstallOpts),
    /// List installed binaries
    List(AppListOpts),
    /// Link installed binaries
    Link(AppLinkOpts),
    /// Unlink installed binaries
    Unlink(AppUnlinkOpts),
    /// List versions of a binary
    Versions(AppVersionsOpts),
    /// Refresh versions of a binary
    Refresh(AppRefreshOpts),
    /// Load cyrene.toml lockfiles in a directory
    Load(AppLoadOpts),
}

#[derive(Args)]
pub struct AppInstallOpts {
    /// Name of app
    apps: Vec<String>,
}
#[derive(Args)]
pub struct AppUpgradeOpts {
    /// Name of app
    apps: Option<Vec<String>>,
}

#[derive(Args)]
pub struct AppUninstallOpts {
    /// Name of app
    apps: Vec<String>,
}
#[derive(Args)]
pub struct AppLinkOpts {
    /// Name of app
    name: String,
    /// Version of app
    version: String,
}
#[derive(Args)]
pub struct AppUnlinkOpts {
    /// Name of app
    name: String,
}

#[derive(Args)]
pub struct AppListOpts {
    /// Long format
    #[arg(short = 'l', long)]
    long: bool,
}
#[derive(Args)]
pub struct AppVersionsOpts {
    /// Name of app
    name: String,
    /// Long format
    #[arg(short = 'l', long)]
    long: bool,
}
#[derive(Args)]
pub struct AppRefreshOpts {
    /// Name of app
    name: Option<String>,
}
#[derive(Args)]
pub struct AppLoadOpts {
    /// Custom path to lockfile
    lockfile: Option<String>,
    /// Use default lockfile
    #[arg(short = 'd', long)]
    default: bool,
}
fn main() -> Result<(), ErrReport> {
    start().into_diagnostic()?;

    Ok(())
}
fn start() -> Result<(), CyreneError> {
    env_logger::init();
    let cli = Cli::parse();

    let mut actions = CyreneManager::new()?;

    match cli.command {
        Commands::Install(app_install_opts) => {
            let app_to_be_installed: Vec<_> =
                app_install_opts.apps.iter().map(AppVersion::from).collect();
            let mut app_actions: Vec<AppVersionAction> = Vec::new();
            for app in app_to_be_installed {
                let install_version = if let Some(ver) = &app.version {
                    if Version::parse(ver).is_ok() {
                        ver.to_string()
                    } else {
                        actions
                            .get_latest_major_release(&app.name, ver.as_str())?
                            .ok_or(CyreneError::AppVersionNotFoundError(
                                ver.to_string(),
                                app.name.clone(),
                            ))?
                    }
                } else {
                    actions.get_latest_version(&app.name)?
                };
                if actions.package_exists(&app.name, &install_version)? {
                    println!(
                        "{} version {} is already installed",
                        &app.name, install_version
                    );
                } else {
                    app_actions.push(AppVersionAction {
                        name: app.name,
                        version: install_version,
                    });
                }
            }

            if !app_actions.is_empty() {
                println!("The following apps will be installed:");
                for app_action in &app_actions {
                    println!("    {}: {}", app_action.name, app_action.version)
                }
                let ans = Confirm::new("Are you sure?").with_default(false).prompt();

                match ans {
                    Ok(true) => {
                        for app_action in app_actions {
                            let linked_version =
                                actions.find_installed_version(&app_action.name)?;
                            println!(
                                "Installing {} version {}",
                                &app_action.name, &app_action.version
                            );
                            actions
                                .install_specific_version(&app_action.name, &app_action.version)?;
                            if let Some(linked_version) = linked_version
                                && is_major_version_equal(&linked_version, &app_action.version)?
                            {
                                actions
                                    .update_lockfile(&app_action.name, Some(&app_action.version))?;
                            }
                            let not_overwritten_exists = actions.link_binaries(
                                &app_action.name,
                                &app_action.version,
                                false,
                            )?;

                            if not_overwritten_exists {
                                println!(
                                    "An existing version is already installed. To use the newly installed binaries, run:"
                                );
                                println!();
                                println!(
                                    "    cyrene link {} {}",
                                    &app_action.name, &app_action.version
                                );
                            };
                        }
                    }
                    Ok(false) => println!("Aborted"),
                    Err(_) => println!("Cannot confirm or deny"),
                }
            } else {
                println!("No action needed");
            }

            Ok(())
        }
        Commands::Link(app_install_opts) => {
            let version = if Version::parse(&app_install_opts.version).is_ok() {
                Some(app_install_opts.version)
            } else {
                actions.find_installed_major_release(
                    &app_install_opts.name,
                    &app_install_opts.version,
                )?
            }
            .ok_or(CyreneError::AppNotInstalledError(
                app_install_opts.name.clone(),
            ))?;
            actions.link_binaries(&app_install_opts.name, &version, true)?;
            actions.update_lockfile(&app_install_opts.name, Some(&version))?;
            Ok(())
        }
        Commands::Unlink(app_install_opts) => {
            println!(
                "Unlinking app binaries for plugin {}",
                &app_install_opts.name
            );
            actions.unlink_binaries(&app_install_opts.name)?;
            Ok(())
        }
        Commands::Refresh(app_version_opts) => {
            if let Some(name) = app_version_opts.name {
                println!("Updating versions database for {}...", &name);
                actions.update_versions(&name)
            } else {
                for app in actions.list_apps()? {
                    println!("Updating versions database for {}...", &app);
                    actions.update_versions(&app)?;
                }
                Ok(())
            }
        }
        Commands::Upgrade(app_install_opts) => app_upgrade(&mut actions, &app_install_opts),
        Commands::Uninstall(app_install_opts) => {
            let app_to_be_installed: Vec<_> =
                app_install_opts.apps.iter().map(AppVersion::from).collect();
            let mut app_actions: Vec<AppVersion> = Vec::new();
            for app in app_to_be_installed {
                let version = match &app.version {
                    Some(version) => {
                        if Version::parse(version).is_ok() {
                            actions.package_exists(&app.name, version.as_str())?;
                            Some(version.to_string())
                        } else {
                            let version = actions
                                .find_installed_major_release(&app.name, version.as_str())?
                                .ok_or(CyreneError::AppVersionNotFoundError(
                                    version.to_string(),
                                    app.name.to_string(),
                                ))?;

                            Some(version)
                        }
                    }
                    None => None,
                };
                app_actions.push(AppVersion {
                    name: app.name,
                    version,
                });
            }
            if !app_actions.is_empty() {
                println!("The following apps will be uninstalled:");
                for app_action in &app_actions {
                    println!(
                        "    {}: {}",
                        app_action.name,
                        match &app_action.version {
                            Some(ver) => ver,
                            None => &"ALL".to_string(),
                        }
                    )
                }
                let ans = Confirm::new("Are you sure?").with_default(false).prompt();

                match ans {
                    Ok(true) => {
                        for app_action in app_actions {
                            match &app_action.version {
                                Some(ver) => {
                                    println!("Uninstalling {} version {}", &app_action.name, ver);
                                    actions.uninstall(&app_action.name, ver)?;
                                }
                                None => {
                                    println!("Uninstalling {}", &app_action.name);
                                    actions.uninstall_all(&app_action.name)?;
                                }
                            };
                        }
                    }
                    Ok(false) => println!("Aborted"),
                    Err(_) => println!("Cannot confirm or deny uninstallation"),
                }
            } else {
                println!("No action needed");
            }
            Ok(())
        }
        Commands::List(app_version_opts) => {
            let lockfile_versions = actions.get_app_version_map()?;
            let apps: Vec<_> = actions
                .list_apps()?
                .iter()
                .flat_map(|f| {
                    let versions = actions.list_installed_app_versions(f).unwrap();
                    let versions: Vec<_> = versions
                        .iter()
                        .map(|f| CyreneAppVersionsAllRow {
                            name: f.name.clone(),
                            version: f.version.clone(),
                            linked: match lockfile_versions.get(&f.name) {
                                Some(ver) => f.version.eq(ver),
                                None => false,
                            },
                        })
                        .collect();
                    versions
                })
                .collect();

            tables::cyrene_app_versions_all(&apps, app_version_opts.long);

            Ok(())
        }
        Commands::Versions(app_version_opts) => {
            let versions: Vec<(String, String)> = actions
                .versions(&app_version_opts.name)?
                .iter()
                .map(|f| (app_version_opts.name.clone(), f.to_string()))
                .collect();

            tables::cyrene_app_versions(&versions, app_version_opts.long);

            Ok(())
        }
        Commands::Load(app_load_opts) => {
            if app_load_opts.default {
                actions.load_lockfile(None)?;
            } else {
                let lockfile_path = if let Some(path) = app_load_opts.lockfile {
                    PathBuf::from(path)
                } else {
                    PathBuf::from("cyrene.toml")
                };
                if !fs::exists(&lockfile_path)? {
                    return Err(CyreneError::LockfileNotFoundError);
                }

                actions.load_lockfile(Some(&lockfile_path))?;
            }

            Ok(())
        }
    }
}

fn app_upgrade(
    actions: &mut CyreneManager,
    app_install_opts: &AppUpgradeOpts,
) -> Result<(), CyreneError> {
    let app_to_be_installed: Vec<_> = if let Some(apps) = &app_install_opts.apps {
        apps.iter().map(AppVersion::from).collect()
    } else {
        actions
            .list_apps()?
            .iter()
            .map(|f| AppVersion {
                name: f.to_string(),
                version: None,
            })
            .collect()
    };
    let mut app_actions: Vec<AppVersionUpgradeAction> = Vec::new();
    for app in app_to_be_installed {
        let old_version = match &app.version {
            Some(ver) => actions.find_installed_major_release(&app.name, ver)?,
            None => actions.find_installed_version(&app.name)?,
        }
        .ok_or(CyreneError::AppNotInstalledError(app.name.to_string()))?;
        let new_version = actions
            .get_latest_major_release(&app.name, &old_version)?
            .ok_or(CyreneError::AppVersionNotFoundError(
                app.name.clone(),
                old_version.clone(),
            ))?;
        if old_version.eq(&new_version) {
            println!("{} is at latest version {}", &app.name, new_version);
        } else {
            app_actions.push(AppVersionUpgradeAction {
                name: app.name,
                old_version,
                new_version,
            })
        }
    }
    if !app_actions.is_empty() {
        println!("The following apps will be upgraded:");
        for app_action in &app_actions {
            println!(
                "    {}: {} -> {}",
                app_action.name, app_action.old_version, app_action.new_version,
            )
        }
        let ans = Confirm::new("Are you sure?").with_default(false).prompt();

        match ans {
            Ok(true) => {
                for app_action in app_actions {
                    println!(
                        "Upgrading {} version {} -> {}",
                        &app_action.name, &app_action.old_version, &app_action.new_version
                    );
                    actions.upgrade(
                        &app_action.name,
                        &app_action.old_version,
                        &app_action.new_version,
                    )?;
                }
            }
            Ok(false) => println!("Aborted"),
            Err(_) => println!("Cannot confirm or deny"),
        }
    } else {
        println!("No action needed");
    }
    Ok(())
}
