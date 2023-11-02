// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::user_profiles::models::UserProfile;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserProfileRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get(&self, jid: &BareJid) -> Result<Option<UserProfile>>;
    async fn set(&self, jid: &BareJid, profile: &UserProfile) -> Result<()>;
    async fn delete(&self, jid: &BareJid) -> Result<()>;

    /// Returns the display name for `jid`. Display name is a cascade of first_name, last_name
    /// and nickname;
    async fn get_display_name(&self, jid: &BareJid) -> Result<Option<String>>;

    async fn clear_cache(&self) -> Result<()>;
}
