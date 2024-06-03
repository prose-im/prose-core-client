// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::TimeDelta;
use jid::BareJid;

use crate::domain::shared::models::MamVersion;

#[derive(Default, Debug, Clone)]
pub struct ServerFeatures {
    pub muc_service: Option<BareJid>,
    pub http_upload_service: Option<HttpUploadService>,
    pub mam_version: Option<MamVersion>,
    pub server_time_offset: TimeDelta,
}

#[derive(Debug, Clone)]
pub struct HttpUploadService {
    pub host: BareJid,
    pub max_file_size: u64,
}
