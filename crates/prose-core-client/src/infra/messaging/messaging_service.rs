// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::delay::Delay;
use xmpp_parsers::message::MessageType;

use prose_xmpp::mods;
use prose_xmpp::stanza::message::mam::ArchivedMessage;
use prose_xmpp::stanza::Message;

use crate::domain::messaging::models::{Emoji, MessageId, StanzaParseError};
use crate::domain::messaging::services::MessagingService;
use crate::domain::shared::models::RoomType;
use crate::dtos::{RoomId, SendMessageRequest};
use crate::infra::xmpp::util::MessageExt;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl MessagingService for XMPPClient {
    async fn send_message(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        request: SendMessageRequest,
    ) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();

        let from = self.connected_jid().ok_or(anyhow::anyhow!(
            "Failed to read the user's JID since the client is not connected."
        ))?;

        let mut message = Message::new()
            .set_type(room_type.message_type())
            .set_id(self.generate_id().into())
            .set_from(from)
            .set_to(room_jid.clone())
            .set_body(request.body.unwrap_or_default())
            .set_chat_state(Some(ChatState::Active))
            .set_markable();
        message.append_attachments(request.attachments);

        chat.send_raw_message(message, true)?;

        Ok(())
    }

    async fn update_message(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        message_id: &MessageId,
        request: SendMessageRequest,
    ) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();

        let from = self.connected_jid().ok_or(anyhow::anyhow!(
            "Failed to read the user's JID since the client is not connected."
        ))?;

        let mut message = Message::new()
            .set_type(room_type.message_type())
            .set_id(self.generate_id().into())
            .set_from(from)
            .set_to(room_jid.clone())
            .set_body(request.body.unwrap_or_default())
            .set_replace(message_id.clone().into_inner().into());
        message.append_attachments(request.attachments);

        chat.send_raw_message(message, true)?;
        Ok(())
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
            emoji.iter().map(|e| e.as_ref().into()),
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

    async fn relay_archived_message_to_room(
        &self,
        room_jid: &RoomId,
        room_type: &RoomType,
        message: ArchivedMessage,
    ) -> Result<()> {
        let timestamp = message
            .forwarded
            .delay
            .ok_or(StanzaParseError::missing_child_node("delay"))?
            .stamp;

        let mut message = *message
            .forwarded
            .stanza
            .ok_or(StanzaParseError::missing_child_node("message"))?;

        let from = message
            .from
            .take()
            .ok_or(StanzaParseError::missing_attribute("from"))?;

        let message = message
            .set_to(room_jid.clone().into_inner())
            .set_type(room_type.message_type())
            .set_delay(Delay {
                from: Some(from),
                stamp: timestamp,
                data: None,
            });

        let chat = self.client.get_mod::<mods::Chat>();
        chat.send_raw_message(message, false)?;

        Ok(())
    }
}

trait RoomMessageType {
    fn message_type(&self) -> MessageType;
}

impl RoomMessageType for RoomType {
    fn message_type(&self) -> MessageType {
        match self {
            RoomType::Unknown => unreachable!("Pending room tried to send a message"),
            RoomType::DirectMessage => MessageType::Chat,
            RoomType::Group
            | RoomType::PrivateChannel
            | RoomType::PublicChannel
            | RoomType::Generic => MessageType::Groupchat,
        }
    }
}
