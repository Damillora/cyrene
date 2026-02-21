use std::{fs, path::PathBuf, sync::Arc};

use crate::{tables::CyreneAppVersionsAllRow, util::is_major_version_equal};
use clap::{Args, Parser, Subcommand};
use console::{Color, Style, style};
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;
use log::debug;
use miette::{ErrReport, IntoDiagnostic};
use semver::Version;

use crate::{
    dirs::CyreneDirs,
    errors::CyreneError,
    lockfile::CyreneLockfileManager,
    manager::CyreneManager,
    transaction::{TransactionCommands, TransactionExecutor},
    versions_cache::CyreneVersionCacheManager,
};

/// App
mod app;
/// Modules for app processing
mod app_module;
/// Directories
mod dirs;
/// Errors
mod errors;
/// Lockfile
mod lockfile;
/// Manager
mod manager;
/// Table models
mod tables;
/// Install transactions
mod transaction;
/// Utilities
mod util;
/// Versions cache
mod versions_cache;

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
    /// Generate environment variables needed by cyrene
    Env,
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
#[tokio::main]
async fn main() -> Result<(), ErrReport> {
    start().await.into_diagnostic()?;

    Ok(())
}
async fn start() -> Result<(), CyreneError> {
    env_logger::init();
    let cli = Cli::parse();

    let dirs = Arc::new(CyreneDirs::default());
    dirs.init_dirs()?;
    let cache_manager = Box::new(CyreneVersionCacheManager::new(&dirs.version_cache_path));
    let lockfile_manager = Box::new(CyreneLockfileManager::new(&dirs.lockfile_path()));

    let actions = Arc::new(CyreneManager::new(
        dirs.clone(),
        lockfile_manager,
        cache_manager,
    ));

    match cli.command {
        Commands::Install(app_install_opts) => {
            let app_to_be_installed: Vec<_> =
                app_install_opts.apps.iter().map(AppVersion::from).collect();
            let mut app_actions: Vec<AppVersionAction> = Vec::new();
            let mut app_actions_unneeded: Vec<AppVersionAction> = Vec::new();
            for app in app_to_be_installed {
                let install_version = if let Some(ver) = &app.version {
                    if Version::parse(ver).is_ok() {
                        ver.to_string()
                    } else {
                        actions
                            .get_latest_major_release(&app.name, ver.as_str())
                            .await?
                            .ok_or(CyreneError::AppVersionNotFound(
                                app.name.clone(),
                                ver.to_string(),
                            ))?
                    }
                } else {
                    actions.get_latest_version(&app.name).await?
                };
                if actions.is_version_installed(&app.name, &install_version)? {
                    app_actions_unneeded.push(AppVersionAction {
                        name: app.name,
                        version: install_version,
                    });
                } else {
                    app_actions.push(AppVersionAction {
                        name: app.name,
                        version: install_version,
                    });
                }
            }
            if !app_actions_unneeded.is_empty() {
                println!();
                tables::cyrene_app_install_unneeded(&app_actions_unneeded);
            }
            if !app_actions.is_empty() {
                println!();
                tables::cyrene_app_install(&app_actions);
                println!();

                let theme = ColorfulTheme {
                    prompt_style: Style::new().fg(Color::Color256(219)),
                    ..Default::default()
                };
                if Confirm::with_theme(&theme)
                    .default(false)
                    .show_default(true)
                    .wait_for_newline(true)
                    .with_prompt(format!(
                        "Proceed with {}?",
                        style("installation").fg(Color::Green).bold()
                    ))
                    .interact()
                    .map_err(CyreneError::Interaction)?
                {
                    let mut transaction = TransactionExecutor::new(actions.clone());
                    for app_action in app_actions.iter() {
                        let linked_version = actions.find_installed_version(&app_action.name)?;
                        transaction.add(TransactionCommands::Install {
                            app: app_action.name.clone(),
                            version: app_action.version.clone(),
                        });
                        if let Some(linked_version) = &linked_version
                            && is_major_version_equal(linked_version, &app_action.version)?
                        {
                            transaction.add(TransactionCommands::LockfileUpdate {
                                app: app_action.name.clone(),
                                version: Some(app_action.version.clone()),
                            });
                        } else if linked_version.is_none() {
                            transaction.add(TransactionCommands::LockfileUpdate {
                                app: app_action.name.clone(),
                                version: Some(app_action.version.clone()),
                            });
                        }
                        transaction.add(TransactionCommands::Link {
                            app: app_action.name.clone(),
                            version: app_action.version.clone(),
                            overwrite: false,
                        });
                    }

                    transaction.execute().await?;
                } else {
                    println!("{}", style("Aborted").fg(console::Color::Red))
                }
            } else {
                println!("{}", style("No action needed").fg(console::Color::Green));
            }

            Ok(())
        }
        Commands::Upgrade(app_install_opts) => app_upgrade(actions, &app_install_opts).await,
        Commands::Uninstall(app_install_opts) => {
            let app_to_be_installed: Vec<_> =
                app_install_opts.apps.iter().map(AppVersion::from).collect();
            let mut app_actions: Vec<AppVersion> = Vec::new();
            for app in app_to_be_installed {
                let version = match &app.version {
                    Some(version) => {
                        if Version::parse(version).is_ok() {
                            actions.is_version_installed(&app.name, version.as_str())?;
                            Some(version.to_string())
                        } else {
                            let version = actions
                                .find_installed_major_release(&app.name, version.as_str())?
                                .ok_or(CyreneError::AppVersionNotFound(
                                    app.name.to_string(),
                                    version.to_string(),
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
                println!();
                tables::cyrene_app_remove(&app_actions);
                println!();
                let theme = ColorfulTheme {
                    prompt_style: Style::new().fg(Color::Color256(219)),
                    ..Default::default()
                };
                if Confirm::with_theme(&theme)
                    .default(false)
                    .show_default(true)
                    .wait_for_newline(true)
                    .with_prompt(format!(
                        "Proceed with {}?",
                        style("uninstallation").fg(Color::Red).bold()
                    ))
                    .interact()
                    .map_err(CyreneError::Interaction)?
                {
                    let mut transaction = TransactionExecutor::new(actions.clone());
                    for app_action in app_actions.iter() {
                        match &app_action.version {
                            Some(ver) => {
                                transaction.add(TransactionCommands::Remove {
                                    app: app_action.name.clone(),
                                    version: ver.clone(),
                                });
                                let current_version = actions
                                    .find_installed_version(&app_action.name)?
                                    .ok_or(CyreneError::AppNotInstalled(
                                        app_action.name.clone(),
                                        "".to_string(),
                                    ))?;
                                let uninstalled_is_linked_version = current_version.eq(ver);
                                if uninstalled_is_linked_version {
                                    debug!(
                                        "App version {} for plugin {} is in use, unlinking app versions",
                                        app_action.name, ver
                                    );
                                    transaction.add(TransactionCommands::Unlink {
                                        app: app_action.name.clone(),
                                    });
                                    let get_release = actions
                                        .find_installed_major_release(&app_action.name, "*")?;
                                    if let Some(get_release) = get_release {
                                        debug!(
                                            "Using latest app versions {} for plugin {} after uninstall",
                                            get_release, &app_action.name
                                        );
                                        transaction.add(TransactionCommands::Link {
                                            app: app_action.name.clone(),
                                            version: ver.clone(),
                                            overwrite: true,
                                        });
                                        transaction.add(TransactionCommands::LockfileUpdate {
                                            app: app_action.name.clone(),
                                            version: Some(get_release),
                                        });
                                    } else {
                                        transaction.add(TransactionCommands::RemoveAll {
                                            app: app_action.name.clone(),
                                        });
                                        transaction.add(TransactionCommands::LockfileUpdate {
                                            app: app_action.name.clone(),
                                            version: None,
                                        });
                                    }
                                }
                            }
                            None => {
                                transaction.add(TransactionCommands::RemoveAll {
                                    app: app_action.name.clone(),
                                });
                                transaction.add(TransactionCommands::Unlink {
                                    app: app_action.name.clone(),
                                });
                                transaction.add(TransactionCommands::LockfileUpdate {
                                    app: app_action.name.clone(),
                                    version: None,
                                });
                            }
                        };
                    }

                    transaction.execute().await?;
                } else {
                    println!("{}", style("Aborted").fg(console::Color::Red));
                }
            } else {
                println!("{}", style("No action needed").fg(console::Color::Green));
            }
            Ok(())
        }
        Commands::Link(app_install_opts) => {
            let version = if Version::parse(&app_install_opts.version).is_ok() {
                Some(app_install_opts.version.clone())
            } else {
                actions.find_installed_major_release(
                    &app_install_opts.name,
                    &app_install_opts.version,
                )?
            }
            .ok_or(CyreneError::AppNotInstalled(
                app_install_opts.name.clone(),
                app_install_opts.version.clone(),
            ))?;
            let mut transaction = TransactionExecutor::new(actions);
            transaction.add(TransactionCommands::Link {
                app: app_install_opts.name.clone(),
                version: version.clone(),
                overwrite: true,
            });
            transaction.add(TransactionCommands::LockfileUpdate {
                app: app_install_opts.name.clone(),
                version: Some(version.clone()),
            });
            transaction.execute().await?;
            Ok(())
        }
        Commands::Unlink(app_install_opts) => {
            let mut transaction = TransactionExecutor::new(actions);
            transaction.add(TransactionCommands::Unlink {
                app: app_install_opts.name.clone(),
            });
            transaction.execute().await?;
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
                            name: f.0.clone(),
                            version: f.1.clone(),
                            linked: match lockfile_versions.get(&f.0) {
                                Some(ver) => f.1.eq(ver),
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
                .versions(&app_version_opts.name)
                .await?
                .iter()
                .map(|f| (app_version_opts.name.clone(), f.to_string()))
                .collect();

            tables::cyrene_app_versions(&versions, app_version_opts.long);

            Ok(())
        }
        Commands::Refresh(app_version_opts) => {
            if let Some(name) = app_version_opts.name {
                println!(
                    "Updating versions database for {}",
                    style(&name).fg(Color::Color256(219)).bold()
                );
                actions.update_versions(&name).await
            } else {
                let list_apps = actions.list_apps()?;
                for name in list_apps.iter() {
                    println!(
                        "Updating versions database for {}",
                        style(&name).fg(Color::Color256(219)).bold()
                    );
                    actions.update_versions(name).await?;
                }
                Ok(())
            }
        }
        Commands::Load(app_load_opts) => {
            let mut transactions = TransactionExecutor::new(actions.clone());
            if app_load_opts.default {
                let txs = actions.load_lockfile(None).await?;
                for tx in txs {
                    transactions.add(tx);
                }
            } else {
                let lockfile_path = if let Some(path) = app_load_opts.lockfile {
                    PathBuf::from(path)
                } else {
                    PathBuf::from("cyrene.toml")
                };
                if !fs::exists(&lockfile_path)
                    .map_err(|e| CyreneError::LockfileLocalRead(lockfile_path.clone(), e))?
                {
                    return Err(CyreneError::LockfileNotFoundError(lockfile_path.clone()));
                }

                let txs = actions.load_lockfile(None).await?;
                for tx in txs {
                    transactions.add(tx);
                }
            }
            transactions.execute().await?;

            Ok(())
        }
        Commands::Env => actions.generate_env(),
    }
}

async fn app_upgrade(
    actions: Arc<CyreneManager>,
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
    let mut app_actions_unneeded: Vec<AppVersionUpgradeAction> = Vec::new();
    for app in app_to_be_installed {
        let old_version = match &app.version {
            Some(ver) => actions.find_installed_major_release(&app.name, ver)?,
            None => actions.find_installed_version(&app.name)?,
        }
        .ok_or(CyreneError::AppNotInstalled(
            app.name.to_string(),
            match &app.version {
                Some(ver) => ver.to_string(),
                None => "latest".to_string(),
            },
        ))?;
        let upgrade_latest = actions.check_upgrade_latest(&app.name)?;
        let new_version = if upgrade_latest {
            actions.get_latest_major_release(&app.name, "*").await?
        } else {
            actions
                .get_latest_major_release(&app.name, &old_version)
                .await?
        }
        .ok_or(CyreneError::AppVersionNotFound(
            app.name.clone(),
            old_version.clone(),
        ))?;
        if old_version.eq(&new_version) {
            app_actions_unneeded.push(AppVersionUpgradeAction {
                name: app.name,
                old_version,
                new_version,
            })
        } else {
            app_actions.push(AppVersionUpgradeAction {
                name: app.name,
                old_version,
                new_version,
            })
        }
    }
    if !app_actions_unneeded.is_empty() {
        println!();
        tables::cyrene_app_upgrade_unneeded(&app_actions_unneeded);
    }
    if !app_actions.is_empty() {
        println!();
        tables::cyrene_app_upgrade(&app_actions);
        println!();
        let theme = ColorfulTheme {
            prompt_style: Style::new().fg(Color::Color256(219)),
            ..Default::default()
        };
        if Confirm::with_theme(&theme)
            .default(false)
            .show_default(true)
            .wait_for_newline(true)
            .with_prompt(format!(
                "Proceed with {}?",
                style("upgrade").fg(Color::Green).bold()
            ))
            .interact()
            .map_err(CyreneError::Interaction)?
        {
            let mut transactions = TransactionExecutor::new(actions.clone());
            for app_action in app_actions.iter() {
                let current_installed = actions.find_installed_version(&app_action.name)?.ok_or(
                    CyreneError::LockfileAppVersion(app_action.name.to_string(), "".to_string()),
                )?;
                let overwrite_installed = current_installed.eq(&app_action.old_version);
                transactions.add(TransactionCommands::Install {
                    app: app_action.name.clone(),
                    version: app_action.new_version.clone(),
                });
                transactions.add(TransactionCommands::Link {
                    app: app_action.name.clone(),
                    version: app_action.new_version.clone(),
                    overwrite: overwrite_installed,
                });
                transactions.add(TransactionCommands::LockfileUpdate {
                    app: app_action.name.clone(),
                    version: Some(app_action.new_version.clone()),
                });
                transactions.add(TransactionCommands::Remove {
                    app: app_action.name.clone(),
                    version: app_action.old_version.clone(),
                });

                transactions.execute().await?;
            }
        } else {
            println!("{}", style("Aborted").fg(console::Color::Red))
        }
    } else {
        println!("{}", style("No action needed").fg(console::Color::Green));
    }
    Ok(())
}
