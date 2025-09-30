use semver::Version;

use crate::errors::CyreneError;

pub fn is_version_equal(old_version: &str, new_version: &str) -> Result<bool, CyreneError> {
    let old_versioning = Version::parse(old_version)?;
    let new_versioning = Version::parse(new_version)?;
    return Ok(old_versioning.eq(&new_versioning));
}
