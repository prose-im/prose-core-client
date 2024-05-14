// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::MamVersion;
use jid::BareJid;

#[derive(Default, Debug)]
pub struct ServerFeatures {
    pub muc_service: Option<BareJid>,
    pub http_upload_service: Option<HttpUploadService>,
    pub mam_version: Option<MamVersion>,
}

#[derive(Debug, Clone)]
pub struct HttpUploadService {
    pub host: BareJid,
    pub max_file_size: u64,
}
