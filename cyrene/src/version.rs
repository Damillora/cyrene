use semver::Version;


pub enum CyreneVersion {
    Semver(Version),
    NonSemver(String),
}
impl CyreneVersion {
    pub fn parse(str: &str) -> Self{
        if let Ok(ver) =  Version::parse(str) {
            CyreneVersion::Semver(ver)
        } else {
            CyreneVersion::NonSemver(str.to_string())
        }
    }
}
impl CyreneVersion {
    pub fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if let CyreneVersion::Semver(self_ver) = self  && let CyreneVersion::Semver(other_ver) = other {
            self_ver.cmp(other_ver)
        } else if let CyreneVersion::NonSemver(self_ver) = self && let CyreneVersion::NonSemver(other_ver) = other {
            self_ver.cmp(other_ver)
        } else if let CyreneVersion::NonSemver(self_ver) = self && let CyreneVersion::Semver(other_ver) = other {
            self_ver.cmp(&other_ver.to_string())
        } else if let CyreneVersion::Semver(self_ver) = self && let CyreneVersion::NonSemver(other_ver) = other {
            self_ver.to_string().cmp(other_ver)
        } else {
            std::cmp::Ordering::Equal
        }
    }
}