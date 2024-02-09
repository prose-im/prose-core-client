// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use prose_xmpp::mods::HttpUpload;

use crate::domain::uploads::models::UploadSlot;
use crate::domain::uploads::services::UploadService;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UploadService for XMPPClient {
    async fn request_upload_slot(
        &self,
        upload_service: &BareJid,
        file_name: &str,
        file_size: u64,
        content_type: &str,
    ) -> Result<UploadSlot> {
        let upload_mod = self.client.get_mod::<HttpUpload>();
        let slot_result = upload_mod
            .request_slot(upload_service, file_name, file_size, Some(content_type))
            .await?;
        Ok(slot_result.try_into()?)
    }
}
