use std::sync::Arc;

use console::{Color, style};

use crate::{errors::CyreneError, manager::CyreneManager};

struct AppActionCommand {
    app: String,
    version: String,
}
enum AppRemoveActionCommand {
    Remove { app: String, version: String },
    RemoveAll { app: String },
}
enum AppFinishActionCommand {
    LockfileUpdate {
        app: String,
        version: Option<String>,
    },
    Link {
        app: String,
        version: String,
        overwrite: bool,
    },
    Unlink {
        app: String,
    },
}
pub enum TransactionCommands {
    Install {
        app: String,
        version: String,
    },
    Remove {
        app: String,
        version: String,
    },
    RemoveAll {
        app: String,
    },
    LockfileUpdate {
        app: String,
        version: Option<String>,
    },
    Link {
        app: String,
        version: String,
        overwrite: bool,
    },
    Unlink {
        app: String,
    },
}

pub struct TransactionExecutor {
    manager: Arc<CyreneManager>,
    // Installation commands are run first
    install: Vec<AppActionCommand>,
    // Then post installs
    post_install: Vec<AppActionCommand>,
    // Then removes (for upgrading)
    remove: Vec<AppRemoveActionCommand>,
    // Update lockfiles here
    finish: Vec<AppFinishActionCommand>,
}

impl TransactionExecutor {
    pub fn new(manager: Arc<CyreneManager>) -> Self {
        Self {
            manager,
            install: Vec::new(),
            post_install: Vec::new(),
            remove: Vec::new(),
            finish: Vec::new(),
        }
    }

    pub fn add(&mut self, cmd: TransactionCommands) {
        match cmd {
            TransactionCommands::Install { app, version } => {
                self.install.push(AppActionCommand {
                    app: app.clone(),
                    version: version.clone(),
                });
                self.post_install.push(AppActionCommand { app, version });
            }
            TransactionCommands::Remove { app, version } => {
                self.remove
                    .push(AppRemoveActionCommand::Remove { app, version });
            }
            TransactionCommands::RemoveAll { app } => {
                self.remove.push(AppRemoveActionCommand::RemoveAll { app });
            }
            TransactionCommands::LockfileUpdate { app, version } => self
                .finish
                .push(AppFinishActionCommand::LockfileUpdate { app, version }),
            TransactionCommands::Link {
                app,
                version,
                overwrite,
            } => self.finish.push(AppFinishActionCommand::Link {
                app,
                version,
                overwrite,
            }),
            TransactionCommands::Unlink { app } => {
                self.finish.push(AppFinishActionCommand::Unlink { app })
            }
        };
    }

    pub async fn execute(&self) -> Result<bool, CyreneError> {
        let install = self.install.iter();
        for install in install {
            println!(
                "Installing {} version {}",
                style(&install.app).fg(Color::Color256(219)).bold(),
                style(&install.version).fg(Color::Green).bold(),
            );
            self.manager
                .install_version(&install.app, &install.version)
                .await?;
        }
        let post_install = self.post_install.iter();
        for post_install in post_install {
            println!(
                "Executing post install commands for {} version {}",
                style(&post_install.app).fg(Color::Color256(219)).bold(),
                style(&post_install.version).fg(Color::Green).bold(),
            );
            self.manager
                .post_install_version(&post_install.app, &post_install.version)
                .await?;
        }
        let remove = self.remove.iter();
        for remove in remove {
            match remove {
                AppRemoveActionCommand::Remove { app, version } => {
                    println!(
                        "Removing {} version {}",
                        style(&app).fg(Color::Color256(219)).bold(),
                        style(&version).fg(Color::Green).bold(),
                    );
                    self.manager.uninstall_version(app, version)?;
                }
                AppRemoveActionCommand::RemoveAll { app } => {
                    println!("Removing {}", style(&app).fg(Color::Color256(219)).bold(),);
                    self.manager.uninstall_all(app)?;
                }
            }
        }
        let finish = self.finish.iter();
        for finish in finish {
            match finish {
                AppFinishActionCommand::LockfileUpdate { app, version } => {
                    let version_string = version.clone().unwrap_or("".to_string());
                    println!(
                        "Updating lockfile for {} version {}",
                        style(&app).fg(Color::Color256(219)).bold(),
                        style(&version_string).fg(Color::Green).bold(),
                    );
                    self.manager.update_lockfile(app, version.as_deref())?;
                }
                AppFinishActionCommand::Link {
                    app,
                    version,
                    overwrite,
                } => {
                    println!(
                        "Linking binaries for {} version {}",
                        style(&app).fg(Color::Color256(219)).bold(),
                        style(&version).fg(Color::Green).bold(),
                    );
                    self.manager.link_binaries(app, version, *overwrite)?;
                }
                AppFinishActionCommand::Unlink { app } => {
                    println!(
                        "Unlinking binaries for {}",
                        style(&app).fg(Color::Color256(219)).bold()
                    );
                    self.manager.unlink_binaries(app)?;
                }
            }
        }
        Ok(true)
    }
}
