// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::settings::models::AccountSettings;

type UpdateHandler = Box<dyn for<'a> FnOnce(&'a mut AccountSettings) + Send>;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait AccountSettingsRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get(&self, jid: &BareJid) -> Result<AccountSettings>;
    async fn update(&self, jid: &BareJid, block: UpdateHandler) -> Result<()>;
    async fn clear_cache(&self) -> Result<()>;
}
