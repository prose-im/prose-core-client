// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::mods::AvatarData;
use prose_xmpp::stanza::avatar;

use crate::domain::user_info::models::AvatarMetadata;

#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserInfoService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_latest_avatar_metadata(&self, from: &BareJid) -> Result<Option<AvatarMetadata>>;
    async fn load_avatar_image(
        &self,
        from: &BareJid,
        image_id: &avatar::ImageId,
    ) -> Result<Option<AvatarData>>;
}
