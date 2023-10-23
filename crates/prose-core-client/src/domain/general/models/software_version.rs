// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

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
