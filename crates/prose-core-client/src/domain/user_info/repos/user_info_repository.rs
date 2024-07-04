// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::shared::models::{AccountId, UserId, UserOrResourceId, UserResourceId};
use crate::domain::user_info::models::{Presence, UserInfo};

pub type UpdateHandler = Box<dyn FnOnce(&mut UserInfo) + Send>;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserInfoRepository: SendUnlessWasm + SyncUnlessWasm {
    /// Tries to resolve `jid` to a FullJid by appending the available resource with the highest
    /// priority. If no available resource is found, returns `jid` as a `Jid`.
    fn resolve_user_id(&self, account: &AccountId, user_id: &UserId) -> Option<UserResourceId>;

    async fn set_user_presence(
        &self,
        account: &AccountId,
        user_id: &UserOrResourceId,
        presence: &Presence,
    ) -> Result<()>;

    async fn get(&self, account: &AccountId, user_id: &UserId) -> Result<Option<UserInfo>>;

    // Upserts `UserInfo` identified by `user_id`. Returns `true` if the `UserInfo` was changed
    // after executing `handler`.
    async fn update(
        &self,
        account: &AccountId,
        user_id: &UserId,
        handler: UpdateHandler,
    ) -> Result<bool>;

    async fn clear_cache(&self, account: &AccountId) -> Result<()>;
}
