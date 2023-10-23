// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::messaging::models::{Message, MessageId, MessageLike};

#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait MessagesRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get(&self, room_id: &BareJid, id: &MessageId) -> Result<Option<Message>>;
    async fn get_all(&self, room_id: &BareJid, ids: &[&MessageId]) -> Result<Vec<Message>>;
    async fn append(&self, room_id: &BareJid, messages: &[&MessageLike]) -> Result<()>;
}
