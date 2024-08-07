// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::stanza::message::mam::ArchivedMessage;

use crate::domain::messaging::models::MessageServerId;
use crate::dtos::RoomId;

#[derive(Debug)]
pub struct MessagePage {
    pub messages: Vec<ArchivedMessage>,
    pub is_last: bool,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait MessageArchiveService: SendUnlessWasm + SyncUnlessWasm {
    /// Returns requested messages in the order from oldest to newest.
    async fn load_messages_before(
        &self,
        room_id: &RoomId,
        before: Option<&MessageServerId>,
        batch_size: u32,
    ) -> Result<MessagePage>;

    /// Returns requested messages in the order from oldest to newest.
    async fn load_messages_after(
        &self,
        room_id: &RoomId,
        after: &MessageServerId,
        batch_size: u32,
    ) -> Result<MessagePage>;

    /// Returns requested messages in the order from oldest to newest.
    async fn load_messages_since(
        &self,
        room_id: &RoomId,
        since: DateTime<Utc>,
        batch_size: u32,
    ) -> Result<MessagePage>;
}
