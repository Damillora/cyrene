use versions::Versioning;

#[derive(Clone)]
pub struct CyreneAppVersions {
    pub name: String,
    pub version: Versioning,
}
