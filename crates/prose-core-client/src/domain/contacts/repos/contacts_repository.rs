// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::contacts::models::Contact;
use crate::domain::shared::models::UserId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait ContactsRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get_all(&self, account_jid: &UserId) -> Result<Vec<Contact>>;
    async fn clear_cache(&self) -> Result<()>;
}
