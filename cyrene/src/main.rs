use clap::{Args, Parser, Subcommand, command};
use directories::ProjectDirs;
use inquire::Confirm;
use miette::{ErrReport, IntoDiagnostic};
use semver::Version;

use crate::{errors::CyreneError, manager::CyreneManager, tables::CyreneAppVersionsRow};
/// Cyrene app definition
pub mod app;
/// Modules used by Cyrene app scripts
pub mod app_module;
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
    name: String,
}
fn main() -> Result<(), ErrReport> {
    start().into_diagnostic()?;

    Ok(())
}
fn start() -> Result<(), CyreneError> {
    env_logger::init();
    let cli = Cli::parse();
    let proj_dirs =
        ProjectDirs::from("com", "Damillora", "Cyrene").ok_or(CyreneError::NoHomeError)?;

    let actions = CyreneManager::new(proj_dirs)?;

    match cli.command {
        Commands::Install(app_install_opts) => {
            if let Some(ver) = &app_install_opts.version {
                if Version::parse(ver).is_ok() {
                    actions.install(&app_install_opts.name, &ver)?;
                } else {
                    let get_release = actions
                        .find_installed_major_release(&app_install_opts.name, ver.as_str())?;

                    if let Some(get_release) = get_release
                        && actions.package_exists(&app_install_opts.name, get_release.as_str())?
                    {
                        app_upgrade(
                            &actions,
                            &AppUpgradeOpts {
                                name: app_install_opts.name,
                                version: app_install_opts.version,
                            },
                        )?;
                        return Ok(());
                    }

                    // Install latest in version req
                    let get_release =
                        actions.get_latest_major_release(&app_install_opts.name, ver.as_str())?;
                    let ans = Confirm::new(
                        format!(
                            "You are going to install {} version {}. Are you sure?",
                            app_install_opts.name, get_release
                        )
                        .as_str(),
                    )
                    .with_default(false)
                    .prompt();

                    match ans {
                        Ok(true) => actions.install(&app_install_opts.name, &get_release)?,
                        Ok(false) => println!("Aborted"),
                        Err(_) => println!("Cannot confirm or deny uninstallation"),
                    }
                }
            } else {
                let latest_release = actions.get_latest_version(&app_install_opts.name)?;
                if actions.package_exists(&app_install_opts.name, latest_release.as_str())? {
                    println!(
                        "Latest {} version {} is installed",
                        &app_install_opts.name, latest_release
                    );
                    return Ok(());
                }
                let ans = Confirm::new(
                    format!(
                        "You are going to install {} version {}. Are you sure?",
                        app_install_opts.name, latest_release,
                    )
                    .as_str(),
                )
                .with_default(false)
                .prompt();

                match ans {
                    Ok(true) => actions.install(&app_install_opts.name, &latest_release)?,
                    Ok(false) => println!("Aborted"),
                    Err(_) => println!("Cannot confirm or deny uninstallation"),
                }
            }
            Ok(())
        }
        Commands::Link(app_install_opts) => {
            let version = if Version::parse(&app_install_opts.version).is_ok() {
                Some(app_install_opts.version)
            } else {
                let get_release = actions.find_installed_major_release(
                    &app_install_opts.name,
                    &app_install_opts.version.as_str(),
                )?;

                get_release
            };
            if let Some(version) = version {
                actions.link_binaries(&app_install_opts.name, &version, true)?;
            } else {
                return Err(CyreneError::AppVersionNotInstalledError);
            }
            Ok(())
        }
        Commands::Unlink(app_install_opts) => {
            actions.unlink_binaries(&app_install_opts.name)?;
            Ok(())
        }
        Commands::Refresh(app_version_opts) => {
            println!(
                "Updating versions database for {}...",
                &app_version_opts.name
            );
            actions.update_versions(&app_version_opts.name)
        }
        Commands::Upgrade(app_install_opts) => app_upgrade(&actions, &app_install_opts),
        Commands::Uninstall(app_install_opts) => match app_install_opts.version {
            Some(version) => {
                let version = if Version::parse(&version).is_ok() {
                    Some(version)
                } else {
                    let get_release = actions
                        .find_installed_major_release(&app_install_opts.name, version.as_str())?;

                    get_release
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
                        Ok(true) => actions.uninstall(&app_install_opts.name, &version)?,
                        Ok(false) => println!("Aborted"),
                        Err(_) => println!("Cannot confirm or deny uninstallation"),
                    }
                } else {
                    return Err(CyreneError::AppVersionNotInstalledError);
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
                    Ok(true) => actions.uninstall_all(&app_install_opts.name)?,
                    Ok(false) => println!("Aborted"),
                    Err(_) => println!("Cannot confirm or deny uninstallation"),
                }
                Ok(())
            }
        },
        Commands::List(app_version_opts) => {
            let apps: Vec<_> = actions
                .list()?
                .iter()
                .map(CyreneAppVersionsRow::from)
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
    }
}

fn app_upgrade(
    actions: &CyreneManager,
    app_install_opts: &AppUpgradeOpts,
) -> Result<(), CyreneError> {
    match &app_install_opts.version {
        Some(version) => {
            let get_release =
                actions.find_installed_major_release(&app_install_opts.name, version.as_str())?;
            if let Some(get_release) = get_release
                && actions.package_exists(&app_install_opts.name, get_release.as_str())?
            {
                println!(
                    "Updating versions database for {}...",
                    &app_install_opts.name
                );
                actions.update_versions(&app_install_opts.name)?;
                let old_version = get_release.as_str();
                let new_version =
                    actions.get_latest_major_release(&app_install_opts.name, old_version)?;
                if util::is_version_equal(old_version, &new_version)? {
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
                    Ok(true) => {
                        actions.upgrade(&app_install_opts.name, &old_version, &new_version)?
                    }
                    Ok(false) => println!("Aborted"),
                    Err(_) => println!("Cannot confirm or deny uninstallation"),
                }
            } else {
                return Err(CyreneError::AppVersionNotInstalledError);
            }
            Ok(())
        }
        None => {
            println!(
                "Updating versions database for {}...",
                &app_install_opts.name
            );
            actions.update_versions(&app_install_opts.name)?;
            let old_version = actions.get_current_version(&app_install_opts.name)?;
            let new_version =
                actions.get_latest_major_release(&app_install_opts.name, &old_version)?;
            if util::is_version_equal(&old_version, &new_version)? {
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
                Err(_) => println!("Cannot confirm or deny uninstallation"),
            }

            Ok(())
        }
    }
}
