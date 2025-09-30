use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, command};
use directories::ProjectDirs;
use inquire::Confirm;

use crate::{errors::CyreneError, manager::CyreneManager};
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
    Upgrade(AppInstallOpts),
    /// Link installed binaries
    Link(AppLinkOpts),
    /// Unlink installed binaries
    Unlink(AppUnlinkOpts),
    /// Uninstall binaries
    Uninstall(AppInstallOpts),
    /// List installed binaries
    List,
    /// List versions of a binary
    Versions(AppVersionsOpts),
}

#[derive(Args)]
pub struct AppInstallOpts {
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
pub struct AppVersionsOpts {
    /// Name of app
    name: String,
    /// Long format
    #[arg(short = 'l', long)]
    long: bool,
}

fn main() -> Result<(), CyreneError> {
    env_logger::init();
    let cli = Cli::parse();
    let proj_dirs =
        ProjectDirs::from("com", "Damillora", "Cyrene").ok_or(CyreneError::NoHomeError)?;

    let actions = CyreneManager::new(proj_dirs)?;

    match cli.command {
        Commands::Install(app_install_opts) => {
            actions.install(&app_install_opts.name, app_install_opts.version.as_deref())?;
            Ok(())
        }
        Commands::Link(app_install_opts) => {
            actions.link_binaries(&app_install_opts.name, &app_install_opts.version, true)?;
            Ok(())
        }
        Commands::Unlink(app_install_opts) => {
            actions.unlink_binaries(&app_install_opts.name)?;
            Ok(())
        }
        Commands::Upgrade(app_install_opts) => match app_install_opts.version {
            Some(version) => {
                if actions.package_exists(&app_install_opts.name, version.as_str())? {
                    let old_version = version.as_str();
                    let new_version =
                        actions.get_latest_version_major(&app_install_opts.name, old_version)?;
                    if util::is_version_equal(old_version, &new_version) {
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
                    return Err(CyreneError::NoAppError);
                }
                Ok(())
            }
            None => {
                let old_version = actions.get_current_version(&app_install_opts.name)?;
                let new_version =
                    actions.get_latest_version_major(&app_install_opts.name, &old_version)?;
                if util::is_version_equal(&old_version, &new_version) {
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

                Ok(())
            }
        },
        Commands::Uninstall(app_install_opts) => match app_install_opts.version {
            Some(version) => {
                if actions.package_exists(&app_install_opts.name, version.as_str())? {
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
                    return Err(CyreneError::NoAppError);
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
        Commands::List => actions.list(),
        Commands::Versions(app_version_opts) => {
            let versions = actions.versions(&app_version_opts.name)?;

            tables::cyrene_app_versions(&versions, app_version_opts.long);

            Ok(())
        }
    }
}
