// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::UploadHeader;
use mime::Mime;
use url::Url;

pub struct UploadSlot {
    pub upload_url: Url,
    pub upload_headers: Vec<UploadHeader>,
    pub download_url: Url,
    pub file_name: String,
    pub media_type: Mime,
    pub file_size: u64,
}
