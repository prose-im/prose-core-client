// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use xmpp_parsers::message::MessageType;

use prose_xmpp::mods;
use prose_xmpp::stanza::message::{ChatState, Emoji};

use crate::domain::messaging::models::MessageId;
use crate::domain::messaging::services::MessagingService;
use crate::domain::shared::models::RoomType;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl MessagingService for XMPPClient {
    async fn send_message(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        body: String,
    ) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.send_message(
            room_jid.clone(),
            body,
            &room_type.message_type(),
            Some(ChatState::Active),
        )
    }

    async fn update_message(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        message_id: &MessageId,
        body: String,
    ) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.update_message(
            message_id.as_ref().into(),
            room_jid.clone(),
            body,
            &room_type.message_type(),
        )
    }

    async fn retract_message(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        message_id: &MessageId,
    ) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.retract_message(
            message_id.as_ref().into(),
            room_jid.clone(),
            &room_type.message_type(),
        )?;
        Ok(())
    }

    async fn react_to_message(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        message_id: &MessageId,
        emoji: &[Emoji],
    ) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.react_to_message(
            message_id.as_ref().into(),
            room_jid.clone(),
            emoji.iter().cloned(),
            &room_type.message_type(),
        )?;
        Ok(())
    }

    async fn set_user_is_composing(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        is_composing: bool,
    ) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.send_chat_state(
            room_jid.clone(),
            if is_composing {
                ChatState::Composing
            } else {
                ChatState::Paused
            },
            &room_type.message_type(),
        )
    }

    async fn send_read_receipt(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        message_id: &MessageId,
    ) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.mark_message_received(
            message_id.as_ref().into(),
            room_jid.clone(),
            &room_type.message_type(),
        )?;
        Ok(())
    }
}

trait RoomMessageType {
    fn message_type(&self) -> MessageType;
}

impl RoomMessageType for RoomType {
    fn message_type(&self) -> MessageType {
        match self {
            RoomType::Pending => unreachable!("Pending room tried to send a message"),
            RoomType::DirectMessage => MessageType::Chat,
            RoomType::Group
            | RoomType::PrivateChannel
            | RoomType::PublicChannel
            | RoomType::Generic => MessageType::Groupchat,
        }
    }
}
