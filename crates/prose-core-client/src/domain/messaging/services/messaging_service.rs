// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::stanza::message::mam::ArchivedMessage;

use crate::domain::messaging::models::{Emoji, MessageId};
use crate::domain::shared::models::{RoomJid, RoomType};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait MessagingService: SendUnlessWasm + SyncUnlessWasm {
    async fn send_message(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        body: String,
    ) -> Result<()>;

    async fn update_message(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        message_id: &MessageId,
        body: String,
    ) -> Result<()>;

    async fn retract_message(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        message_id: &MessageId,
    ) -> Result<()>;

    async fn react_to_message(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        message_id: &MessageId,
        emoji: &[Emoji],
    ) -> Result<()>;

    async fn set_user_is_composing(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        is_composing: bool,
    ) -> Result<()>;

    async fn send_read_receipt(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        message_id: &MessageId,
    ) -> Result<()>;

    async fn relay_archived_message_to_room(
        &self,
        room_jid: &RoomJid,
        room_type: &RoomType,
        message: ArchivedMessage,
    ) -> Result<()>;
}
