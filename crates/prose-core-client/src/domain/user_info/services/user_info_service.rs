// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::mods::AvatarData;
use prose_xmpp::RequestError;

use crate::domain::shared::models::{AvatarId, BareEntityId, ParticipantIdRef, UserResourceId};
use crate::domain::user_info::models::{UserMetadata, UserProfile};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserInfoService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_avatar_image(
        &self,
        from: &BareEntityId,
        image_id: &AvatarId,
    ) -> Result<Option<AvatarData>, RequestError>;

    async fn load_vcard_temp(
        &self,
        from: ParticipantIdRef<'_>,
    ) -> Result<Option<UserProfile>, RequestError>;

    async fn load_user_metadata(
        &self,
        from: &UserResourceId,
        now: DateTime<Utc>,
    ) -> Result<Option<UserMetadata>, RequestError>;
}
