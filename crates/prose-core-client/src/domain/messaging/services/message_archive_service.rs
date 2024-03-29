// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::stanza::message::mam::ArchivedMessage;

use crate::domain::messaging::models::StanzaId;
use crate::dtos::RoomId;

pub struct MessagePage {
    pub messages: Vec<ArchivedMessage>,
    pub is_last: bool,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait MessageArchiveService: SendUnlessWasm + SyncUnlessWasm {
    /// Returns requested messages in the order from oldest to newest.
    async fn load_messages(
        &self,
        room_jid: &RoomId,
        before: Option<&StanzaId>,
        after: Option<&StanzaId>,
        batch_size: u32,
    ) -> Result<MessagePage>;
}
