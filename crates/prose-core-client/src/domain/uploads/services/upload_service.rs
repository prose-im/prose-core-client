// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use mime::Mime;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::uploads::models::UploadSlot;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UploadService: SendUnlessWasm + SyncUnlessWasm {
    async fn request_upload_slot(
        &self,
        upload_service: &BareJid,
        file_name: &str,
        file_size: u64,
        media_type: &Mime,
    ) -> Result<UploadSlot>;
}
