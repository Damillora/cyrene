use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum CyreneError {
    #[error("Unable to read from version cache: {0}")]
    VersionCacheRead(std::io::Error),
    #[error("Unable to write from version cache: {0}")]
    VersionCacheWrite(std::io::Error),
    #[error("Unable to deserialize version cache: {0}")]
    VersionCacheDeserialize(toml::de::Error),
    #[error("Unable to serialize version cache: {0}")]
    VersionCacheSerialize(toml::ser::Error),
    #[error("Unable to read lockfile: {0}")]
    LockfileRead(std::io::Error),
    #[error("Unable to write lockfile: {0}")]
    LockfileWrite(std::io::Error),
    #[error("Unable to deserialize lockfile: {0}")]
    LockfileDeserialize(toml::de::Error),
    #[error("Unable to serialize lockfile: {0}")]
    LockfileSerialize(toml::ser::Error),
    #[error("Unable to load lockfile from {0}: {1}")]
    LockfileLocalRead(PathBuf, std::io::Error),
    #[error("Unable to initialize directory {0}: {1}")]
    DirectoryInit(PathBuf, std::io::Error),
    #[error("Unable to parse app: {0}")]
    AppDeserialize(toml::de::Error),
    #[error("Unable to read app from {0}: {1}")]
    AppRead(PathBuf, std::io::Error),
    #[error("Unable to parse version string {0}: {1}")]
    VersionParse(String, semver::Error),
    #[error("Unable to fetch version info from {0}: {1}")]
    VersionFetch(String, reqwest::Error),
    #[error("Unable to execute JSON query {0}: {1}")]
    VersionQueryParse(String, jsonpath_rust::parser::errors::JsonPathError),
    #[error("Unable to list apps in {0}: {1}")]
    AppList(PathBuf, std::io::Error),
    #[error("Unable to find app {0} version {1}")]
    AppVersionNotFound(String, String),
    #[error("Interaction error: {0}")]
    Interaction(dialoguer::Error),
    #[error("Unable to confirm the installation of {0} version {1}: {2}")]
    AppCheck(String, String, std::io::Error),
    #[error("Unable to find installation of {0} version {1}")]
    AppNotInstalled(String, String),
    #[error("Versions for {0} not cached")]
    AppVersionNotCached(String),
    #[error("Unable to create installation directory for {0} version {1}: {2}")]
    AppInstallDirCreateError(String, String, std::io::Error),
    #[error("Unable to download from {0}: {1}")]
    Download(String, reqwest::Error),
    #[error("Unable to save download from {0}: {1}")]
    DownloadWrite(String, std::io::Error),
    #[error("Somehow unable to access the current executable")]
    ExeCheck(std::io::Error),
    #[error("Cyrene was about to close the causality loop")]
    AppLinkingToSelf,
    #[error("Unable to read link in {0}: {1}")]
    AppLinkRead(String, std::io::Error),
    #[error("Unable to remove link in {0}: {1}")]
    AppLinkRemove(String, std::io::Error),
    #[error("Unable to link {0} to {1}: {1}")]
    AppLinkCreate(String, String, std::io::Error),
    #[error("Cannot find lockfile at {0}")]
    LockfileNotFoundError(PathBuf),
    #[error("Unable to remove {0} version {1}: {2}")]
    AppRemove(String, String, std::io::Error),
    #[error("Non existent app {0} version {1} in lockfile")]
    LockfileAppVersion(String, String),
    #[error("Unable to read config: {0}")]
    ConfigRead(std::io::Error),
    #[error("Unable to write config: {0}")]
    ConfigWrite(std::io::Error),
    #[error("Unable to deserialize config: {0}")]
    ConfigDeserialize(toml::de::Error),
    #[error("Unable to serialize config: {0}")]
    ConfigSerialize(toml::ser::Error),
}
