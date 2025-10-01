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
    #[error("IO error")]
    FsError(#[from] std::io::Error),
    #[error("Cannot create context")]
    RuneContextError(#[from] ContextError),
    #[error("Cannot fetch source")]
    RuneSourceError(#[from] FromPathError),
    #[error("Cannot display error messages")]
    RuneEmitError(#[from] EmitError),
    #[error("Cannot parse scripts")]
    RuneBuildError(#[from] BuildError),
    #[error("Error while running script")]
    RuneRuntimeError(#[from] RuntimeError),
    #[error("Error while running script")]
    RuneVmError(#[from] VmError),
    #[error("Cannot allocate runtime")]
    RuneAllocError(#[from] rune::alloc::Error),
    #[error("Cannot find cyrene configuration")]
    NoHomeError,
    #[error("App for plugin {0} is not installed")]
    AppNotInstalledError(String),
    #[error("App not registered in lockfile")]
    AppVersionNotInLockfileError,
    #[error("App version {0} for plugin {1} is not installed")]
    AppVersionNotInstalledError(String, String),
    #[error("Cannot find app version {0} for plugin {1} in version list")]
    AppVersionNotFoundError(String, String),
    #[error("Cannot find app version {0} for plugin {1} in versions cache")]
    AppVersionNotFoundInCacheError(String, String),
    #[error("Cannot find app versions for plugin {0} in versions cache")]
    AppVersionNotInCacheError(String),
    #[error("Error parsing versions")]
    VersionError(#[from] semver::Error),
    #[error("Cannot locate cyrene")]
    ExePathError,
    #[error("Cannot locate plugin")]
    PluginPathError,
    #[error("Cannot query the web")]
    HttpError(#[from] reqwest::Error),
    #[error("Lockfile is malformed")]
    LockfileReadError(#[from] toml::de::Error),
    #[error("Cannot find lockfile")]
    LockfileNotFoundError,
    #[error("App in lockfile is invalid")]
    LockfileAppVersionError,
    #[error("Cannot write lockfile")]
    LockfileWriteError(#[from] toml::ser::Error),
}
