use rune::{
    BuildError, ContextError,
    diagnostics::EmitError,
    runtime::{RuntimeError, VmError},
    source::FromPathError,
};
use thiserror::Error;

use miette::Diagnostic;

#[derive(Error, Diagnostic, Debug)]
pub enum CyreneError {
    #[error("IO error: {0}")]
    FsError(#[from] std::io::Error),
    #[error("Cannot create context: {0}")]
    RuneContextError(#[from] ContextError),
    #[error("Cannot fetch source: {0}")]
    RuneSourceError(#[from] FromPathError),
    #[error("Cannot display error messages: {0}")]
    RuneEmitError(#[from] EmitError),
    #[error("Cannot parse scripts: {0}")]
    RuneBuildError(#[from] BuildError),
    #[error("Error while running script: {0}")]
    RuneRuntimeError(#[from] RuntimeError),
    #[error("Error while running script: {0}")]
    RuneVmError(#[from] VmError),
    #[error("Cannot allocate runtime: {0}")]
    RuneAllocError(#[from] rune::alloc::Error),
    #[error("Cannot find cyrene configuration")]
    NoHomeError,
    #[error("App for plugin {0} is not installed")]
    AppNotInstalledError(String),
    #[error("App for plugin {0} is not registered in lockfile")]
    AppNotInLockfileError(String),
    #[error("App version {0} for plugin {1} is not installed")]
    AppVersionNotInstalledError(String, String),
    #[error("Cannot find app version {0} for plugin {1} in version list")]
    AppVersionNotFoundError(String, String),
    #[error("Cannot find app version {0} for plugin {1} in versions cache")]
    AppVersionNotFoundInCacheError(String, String),
    #[error("Cannot find app versions for plugin {0} in versions cache")]
    AppVersionNotInCacheError(String),
    #[error("Error parsing versions: {0}")]
    VersionError(#[from] semver::Error),
    #[error("Cannot locate cyrene")]
    ExePathError,
    #[error("Cannot locate plugin")]
    PluginPathError,
    #[error("Cannot query the web: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Lockfile is malformed: {0}")]
    LockfileReadError(#[from] toml::de::Error),
    #[error("Cannot find lockfile")]
    LockfileNotFoundError,
    #[error("App version {0} for plugin {1} in lockfile is invalid")]
    LockfileAppVersionError(String, String),
    #[error("Plugin {0} in lockfile is invalid")]
    LockfileAppError(String),
    #[error("Cannot write lockfile: {0}")]
    LockfileWriteError(#[from] toml::ser::Error),
    #[error("Console is interrupted: {0}")]
    ConsoleInterruptedError(#[from] dialoguer::Error),
    #[error("Cyrene was about to sacrifice itself to the Remembrance")]
    AppLinkingToItselfError,
}
