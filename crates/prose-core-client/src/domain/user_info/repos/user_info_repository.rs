// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::shared::models::{UserId, UserOrResourceId, UserResourceId};
use crate::domain::user_info::models::{AvatarMetadata, Presence, UserInfo, UserStatus};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserInfoRepository: SendUnlessWasm + SyncUnlessWasm {
    /// Tries to resolve `jid` to a FullJid by appending the available resource with the highest
    /// priority. If no available resource is found, returns `jid` as a `Jid`.
    fn resolve_user_id_to_user_resource_id(&self, jid: &UserId) -> Option<UserResourceId>;

    async fn get_user_info(&self, jid: &UserId) -> Result<Option<UserInfo>>;

    async fn set_avatar_metadata(&self, jid: &UserId, metadata: &AvatarMetadata) -> Result<()>;
    async fn set_user_activity(
        &self,
        jid: &UserId,
        user_activity: Option<&UserStatus>,
    ) -> Result<()>;
    async fn set_user_presence(&self, jid: &UserOrResourceId, presence: &Presence) -> Result<()>;

    async fn clear_cache(&self) -> Result<()>;
}
