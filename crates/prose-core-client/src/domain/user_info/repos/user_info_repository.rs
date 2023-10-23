// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::{BareJid, Jid};
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::user_info::models::{AvatarMetadata, Presence, UserActivity, UserInfo};

#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserInfoRepository: SendUnlessWasm + SyncUnlessWasm {
    /// Tries to resolve `jid` to a FullJid by appending the available resource with the highest
    /// priority. If no available resource is found, returns `jid` as a `Jid`.
    fn resolve_bare_jid_to_full(&self, jid: &BareJid) -> Jid;

    async fn get_user_info(&self, jid: &BareJid) -> Result<Option<UserInfo>>;

    async fn set_avatar_metadata(&self, jid: &BareJid, metadata: &AvatarMetadata) -> Result<()>;
    async fn set_user_activity(
        &self,
        jid: &BareJid,
        user_activity: Option<&UserActivity>,
    ) -> Result<()>;
    async fn set_user_presence(&self, jid: &Jid, presence: &Presence) -> Result<()>;
}
