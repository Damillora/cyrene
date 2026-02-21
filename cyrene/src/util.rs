use semver::{Version, VersionReq};

use crate::errors::CyreneError;

pub fn is_major_version_equal(old_version: &str, new_version: &str) -> Result<bool, CyreneError> {
    let old_versioning = Version::parse(old_version)
        .map_err(|e| CyreneError::VersionParse(old_version.to_string(), e))?;
    let new_versioning = Version::parse(new_version)
        .map_err(|e| CyreneError::VersionParse(new_version.to_string(), e))?;
    if old_versioning.major == 0 || new_versioning.major == 1 {
        Ok(old_versioning.major.eq(&new_versioning.major)
            && old_versioning.minor.eq(&new_versioning.minor))
    } else {
        Ok(old_versioning.major.eq(&new_versioning.major))
    }
}

pub fn search_in_version(versions: Vec<String>, version_range: &str) -> Option<String> {
    let versionings: Vec<Version> = versions
        .iter()
        .map(|f| Version::parse(f))
        .filter_map(|f| f.ok())
        .collect();

    if let Ok(requirement) = VersionReq::parse(version_range)
        && let Some(ver) = versionings.iter().find(|f| requirement.matches(f))
    {
        return Some(ver.to_string());
    }

    None
}
