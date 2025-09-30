use rune::{
    BuildError, ContextError,
    diagnostics::EmitError,
    runtime::{RuntimeError, VmError},
    source::FromPathError,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CyreneError {
    #[error("FS error")]
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
    #[error("Cannot find app")]
    NoAppError,
    #[error("Something is wrong with versioning")]
    VersioningError,
    #[error("Error parsing versions")]
    VersionError(#[from] semver::Error),
    #[error("Cannot locate cyrene")]
    ExePathError,
    #[error("Cannot locate plugin")]
    PluginPathError,
    #[error("Cannot query the web")]
    HttpError(#[from] reqwest::Error),
    #[error("Cannot read lockfile")]
    LockfileReadError(#[from] toml::de::Error),
    #[error("Cannot read lockfile correctly")]
    LockfileError,
    #[error("Cannot write lockfile")]
    LockfileWriteError(#[from] toml::ser::Error),
}
