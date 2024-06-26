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
    /// Does the server support vCard4?
    pub vcard4: bool,
    /// Does the server support XEP-0398: User Avatar to vCard-Based Avatars Conversion?
    pub avatar_pep_vcard_conversion: bool,
    /// The offset between our local time and the server's time.
    pub server_time_offset: TimeDelta,
}

#[derive(Debug, Clone)]
pub struct HttpUploadService {
    pub host: BareJid,
    pub max_file_size: u64,
}
