use semver::Version;

use crate::errors::CyreneError;

pub fn is_major_version_equal(old_version: &str, new_version: &str) -> Result<bool, CyreneError> {
    let old_versioning = Version::parse(old_version)?;
    let new_versioning = Version::parse(new_version)?;
    if old_versioning.major == 0 || new_versioning.major == 1 {
        Ok(old_versioning.major.eq(&new_versioning.major)
            && old_versioning.minor.eq(&new_versioning.minor))
    } else {
        Ok(old_versioning.major.eq(&new_versioning.major))
    }
}
