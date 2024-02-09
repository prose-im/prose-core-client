// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct UploadSlot {
    pub upload_url: Url,
    pub upload_headers: Vec<UploadHeader>,
    pub download_url: Url,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UploadHeader {
    pub name: String,
    pub value: String,
}

impl UploadHeader {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}
