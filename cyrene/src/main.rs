use std::{fs, path::PathBuf};

use clap::{Args, Parser, Subcommand, command};
use inquire::Confirm;
use miette::{ErrReport, IntoDiagnostic};
use semver::Version;

use crate::{errors::CyreneError, manager::CyreneManager, tables::CyreneAppVersionsRow};
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
    /// Link installed binaries
    Link(AppLinkOpts),
    /// Unlink installed binaries
    Unlink(AppUnlinkOpts),
    /// Uninstall binaries
    Uninstall(AppUninstallOpts),
    /// List installed binaries
    List(AppListOpts),
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
    name: String,
    /// Version of app
    version: Option<String>,
}
#[derive(Args)]
pub struct AppUpgradeOpts {
    /// Name of app
    name: String,
    /// Version of app
    version: Option<String>,
}
#[derive(Args)]
pub struct AppUninstallOpts {
    /// Name of app
    name: String,
    /// Version of app
    version: Option<String>,
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
            let install_version = if let Some(ver) = &app_install_opts.version {
                if Version::parse(ver).is_ok() {
                    ver.to_string()
                } else {
                    actions
                        .get_latest_major_release(&app_install_opts.name, ver.as_str())?
                        .ok_or(CyreneError::AppVersionNotFoundError(
                            ver.to_string(),
                            app_install_opts.name.clone(),
                        ))?
                }
            } else {
                actions.get_latest_version(&app_install_opts.name)?
            };

            if actions.package_exists(&app_install_opts.name, &install_version)? {
                println!(
                    "{} version {} is already installed",
                    &app_install_opts.name, install_version
                );
            } else {
                let ans = Confirm::new(
                    format!(
                        "You are going to install {} version {}. Are you sure?",
                        app_install_opts.name, &install_version
                    )
                    .as_str(),
                )
                .with_default(false)
                .prompt();

                match ans {
                    Ok(true) => {
                        println!(
                            "Installing {} version {}",
                            &app_install_opts.name, &install_version
                        );
                        actions
                            .install_specific_version(&app_install_opts.name, &install_version)?;
                        actions.update_lockfile(&app_install_opts.name, &install_version)?;
                        let not_overwritten_exists = actions.link_binaries(
                            &app_install_opts.name,
                            &install_version,
                            false,
                        )?;

                        if not_overwritten_exists {
                            println!(
                                "An existing version is already installed. To use the newly installed binaries, run:"
                            );
                            println!();
                            println!(
                                "    cyrene link {} {}",
                                &app_install_opts.name, &install_version
                            );
                        };
                    }
                    Ok(false) => println!("Aborted"),
                    Err(_) => println!("Cannot confirm or deny"),
                }
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
            actions.update_lockfile(&app_install_opts.name, &version)?;
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
        Commands::Uninstall(app_install_opts) => match app_install_opts.version {
            Some(version) => {
                let version = if Version::parse(&version).is_ok() {
                    Some(version)
                } else {
                    actions
                        .find_installed_major_release(&app_install_opts.name, version.as_str())?
                };
                if let Some(version) = version
                    && actions.package_exists(&app_install_opts.name, version.as_str())?
                {
                    let ans = Confirm::new(
                        format!(
                            "You are going to uninstall {} version {}. Are you sure?",
                            app_install_opts.name, version
                        )
                        .as_str(),
                    )
                    .with_default(false)
                    .prompt();

                    match ans {
                        Ok(true) => {
                            actions.uninstall(&app_install_opts.name, &version)?;
                            println!("Uninstalled {} version {}", &app_install_opts.name, version);
                        }
                        Ok(false) => println!("Aborted"),
                        Err(_) => println!("Cannot confirm or deny uninstallation"),
                    }
                } else {
                    return Err(CyreneError::AppNotInstalledError(
                        app_install_opts.name.clone(),
                    ));
                }
                Ok(())
            }
            None => {
                let ans = Confirm::new(
                    format!(
                        "You are going to uninstall ALL versions of {}! Are you sure?",
                        app_install_opts.name
                    )
                    .as_str(),
                )
                .with_default(false)
                .prompt();

                match ans {
                    Ok(true) => {
                        actions.uninstall_all(&app_install_opts.name)?;
                        println!("Uninstalled {}", &app_install_opts.name);
                    }
                    Ok(false) => println!("Aborted"),
                    Err(_) => println!("Cannot confirm or deny uninstallation"),
                }
                Ok(())
            }
        },
        Commands::List(app_version_opts) => {
            let apps: Vec<_> = actions
                .list_apps()?
                .iter()
                .flat_map(|f| {
                    let versions = actions.list_installed_app_versions(f).unwrap();
                    let versions: Vec<_> =
                        versions.iter().map(CyreneAppVersionsRow::from).collect();
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
    let old_version = match &app_install_opts.version {
        Some(ver) => actions.find_installed_major_release(&app_install_opts.name, ver)?,
        None => actions.find_installed_version(&app_install_opts.name)?,
    }
    .ok_or(CyreneError::AppNotInstalledError(
        app_install_opts.name.clone(),
    ))?;
    let new_version = actions
        .get_latest_major_release(&app_install_opts.name, &old_version)?
        .ok_or(CyreneError::AppVersionNotFoundError(
            app_install_opts.name.clone(),
            old_version.clone(),
        ))?;
    if old_version.eq(&new_version) {
        println!(
            "{} is at latest version {}",
            &app_install_opts.name, new_version
        );
        return Ok(());
    }
    let ans = Confirm::new(
        format!(
            "You are going to upgrade {} version {} to {}. Are you sure?",
            app_install_opts.name, old_version, new_version
        )
        .as_str(),
    )
    .with_default(false)
    .prompt();

    match ans {
        Ok(true) => actions.upgrade(&app_install_opts.name, &old_version, &new_version)?,
        Ok(false) => println!("Aborted"),
        Err(_) => println!("Cannot confirm or deny"),
    }
    Ok(())
}
