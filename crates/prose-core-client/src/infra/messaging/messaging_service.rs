// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::delay::Delay;
use xmpp_parsers::message::MessageType;

use prose_xmpp::mods;
use prose_xmpp::stanza::message::mam::ArchivedMessage;
use prose_xmpp::stanza::Message;

use crate::domain::messaging::models::{
    Emoji, MessageId, SendMessageRequest, StanzaId, StanzaParseError,
};
use crate::domain::messaging::services::MessagingService;
use crate::dtos::{MucId, RoomId, UserId};
use crate::infra::xmpp::util::MessageExt;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl MessagingService for XMPPClient {
    async fn send_message(&self, room_id: &RoomId, request: SendMessageRequest) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();

        let from = self.connected_jid().ok_or(anyhow::anyhow!(
            "Failed to read the user's JID since the client is not connected."
        ))?;

        let mut message = Message::new()
            .set_type(room_id.message_type())
            .set_id(request.id.into_inner().into())
            .set_from(from)
            .set_to(room_id.clone().into_bare())
            .set_message_body(request.body)
            .set_chat_state(Some(ChatState::Active))
            .set_markable()
            .set_store(true);
        message.append_attachments(request.attachments);

        chat.send_raw_message(message, true)?;

        Ok(())
    }

    async fn update_message(
        &self,
        room_id: &RoomId,
        message_id: &MessageId,
        request: SendMessageRequest,
    ) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();

        let from = self.connected_jid().ok_or(anyhow::anyhow!(
            "Failed to read the user's JID since the client is not connected."
        ))?;

        let mut message = Message::new()
            .set_type(room_id.message_type())
            .set_id(request.id.into_inner().into())
            .set_from(from)
            .set_to(room_id.clone().into_bare())
            .set_message_body(request.body)
            .set_replace(message_id.clone().into_inner().into())
            .set_store(true);
        message.append_attachments(request.attachments);

        chat.send_raw_message(message, true)?;
        Ok(())
    }

    async fn retract_message(&self, room_id: &RoomId, message_id: &MessageId) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.retract_message(
            message_id.as_ref().into(),
            room_id.clone().into_bare(),
            &room_id.message_type(),
        )?;
        Ok(())
    }

    async fn react_to_chat_message(
        &self,
        room_id: &UserId,
        message_id: &MessageId,
        emoji: &[Emoji],
    ) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.react_to_chat_message(
            message_id.as_ref().into(),
            room_id.clone().into_inner(),
            emoji.iter().map(|e| e.as_ref().into()),
        )?;
        Ok(())
    }

    async fn react_to_muc_message(
        &self,
        room_id: &MucId,
        message_id: &StanzaId,
        emoji: &[Emoji],
    ) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.react_to_muc_message(
            message_id.as_ref().into(),
            room_id.clone().into_inner(),
            emoji.iter().map(|e| e.as_ref().into()),
        )?;
        Ok(())
    }

    async fn set_user_is_composing(&self, room_id: &RoomId, is_composing: bool) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.send_chat_state(
            room_id.clone().into_bare(),
            if is_composing {
                ChatState::Composing
            } else {
                ChatState::Paused
            },
            &room_id.message_type(),
        )
    }

    async fn send_read_receipt(&self, room_id: &RoomId, message_id: &MessageId) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.mark_message_received(
            message_id.as_ref().into(),
            room_id.clone().into_bare(),
            &room_id.message_type(),
        )?;
        Ok(())
    }

    async fn relay_archived_message_to_room(
        &self,
        room_id: &RoomId,
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
            .set_to(room_id.clone().into_bare())
            .set_type(room_id.message_type())
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

impl RoomMessageType for RoomId {
    fn message_type(&self) -> MessageType {
        match self {
            RoomId::User(_) => MessageType::Chat,
            RoomId::Muc(_) => MessageType::Groupchat,
        }
    }
}
