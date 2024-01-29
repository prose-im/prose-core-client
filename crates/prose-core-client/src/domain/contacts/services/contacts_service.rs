// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::contacts::models::Contact;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait ContactsService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_contacts(&self, account_jid: &BareJid) -> Result<Vec<Contact>>;
    async fn add_contact(&self, jid: &BareJid) -> Result<()>;
    async fn remove_contact(&self, jid: &BareJid) -> Result<()>;
}
