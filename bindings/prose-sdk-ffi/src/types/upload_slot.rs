// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::Url;
use prose_core_client::dtos::{UploadHeader as CoreUploadHeader, UploadSlot as CoreUploadSlot};

#[derive(uniffi::Record)]
pub struct UploadSlot {
    pub upload_url: Url,
    pub upload_headers: Vec<UploadHeader>,
    pub download_url: Url,
    pub media_type: String,
    pub file_size: u64,
    pub file_name: String,
}

#[derive(uniffi::Record)]
pub struct UploadHeader {
    pub name: String,
    pub value: String,
}

impl From<CoreUploadSlot> for UploadSlot {
    fn from(value: CoreUploadSlot) -> Self {
        UploadSlot {
            upload_url: value.upload_url.into(),
            upload_headers: value.upload_headers.into_iter().map(Into::into).collect(),
            download_url: value.download_url.into(),
            file_name: value.file_name,
            media_type: value.media_type.to_string(),
            file_size: value.file_size,
        }
    }
}

impl From<CoreUploadHeader> for UploadHeader {
    fn from(value: CoreUploadHeader) -> Self {
        UploadHeader {
            name: value.name,
            value: value.value,
        }
    }
}
