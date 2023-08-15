use xmpp_parsers::version::VersionResult;

#[derive(Debug, Clone, PartialEq)]
pub struct SoftwareVersion {
    pub name: String,
    pub version: String,
    pub os: Option<String>,
}

impl Default for SoftwareVersion {
    fn default() -> Self {
        SoftwareVersion {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            os: None,
        }
    }
}

impl From<SoftwareVersion> for VersionResult {
    fn from(value: SoftwareVersion) -> Self {
        VersionResult {
            name: value.name,
            version: value.version,
            os: value.os,
        }
    }
}
