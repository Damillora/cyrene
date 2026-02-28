use semver::{Version, VersionReq};

use crate::{errors::CyreneError, version::CyreneVersion};

pub fn is_major_version_equal(old_version: &str, new_version: &str) -> Result<bool, CyreneError> {
    let old_versioning = CyreneVersion::parse(old_version);
    let new_versioning = CyreneVersion::parse(new_version);
    if let CyreneVersion::Semver(old_versioning) = old_versioning
        && let CyreneVersion::Semver(new_versioning) = new_versioning
    {
        if old_versioning.major == 0 || new_versioning.major == 1 {
            Ok(old_versioning.major.eq(&new_versioning.major)
                && old_versioning.minor.eq(&new_versioning.minor))
        } else {
            Ok(old_versioning.major.eq(&new_versioning.major))
        }
    } else {
        Ok(old_version.eq(new_version))
    }
}

pub fn search_in_version(
    semver: bool,
    versions: Vec<String>,
    version_range: &str,
) -> Option<String> {
    if semver {
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
    } else if let Some(ver) = versions.iter().find(|e| e.as_str() == version_range) {
        return Some(ver.clone());
    }

    None
}
