// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::Room;
use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::{ConnectedRoom, Message, MessageId};
use anyhow::Result;
use jid::BareJid;
use prose_xmpp::mods;
use prose_xmpp::stanza::message;
use prose_xmpp::stanza::message::{ChatState, Emoji};
use std::sync::Arc;

pub struct Generic {}

impl<Kind, D: DataCache, A: AvatarCache> Room<Kind, D, A> {
    pub fn to_generic(&self) -> Room<Generic, D, A> {
        Room {
            inner: self.inner.clone(),
            inner_mut: self.inner_mut.clone(),
            to_connected_room: Arc::new(|room| Ok(ConnectedRoom::Generic(room))),
            _type: Default::default(),
        }
    }
}

impl<Kind, D: DataCache, A: AvatarCache> Room<Kind, D, A> {
    pub fn jid(&self) -> &BareJid {
        &self.inner.jid
    }

    pub fn name(&self) -> Option<&str> {
        self.inner.name.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.inner.description.as_deref()
    }

    pub fn user_nickname(&self) -> &str {
        &self.inner.user_nickname
    }

    pub fn subject(&self) -> Option<String> {
        self.inner_mut.read().subject.clone()
    }

    pub fn members(&self) -> &[BareJid] {
        self.inner.members.as_slice()
    }
}

impl<Kind, D: DataCache, A: AvatarCache> Room<Kind, D, A> {
    pub async fn send_message(&self, body: impl Into<String>) -> Result<()> {
        let chat = self.inner.xmpp.get_mod::<mods::Chat>();
        chat.send_message(
            self.inner.jid.clone(),
            body,
            &self.inner.message_type,
            Some(ChatState::Active),
        )
    }

    pub async fn update_message(&self, id: MessageId, body: impl Into<String>) -> Result<()> {
        let chat = self.inner.xmpp.get_mod::<mods::Chat>();
        chat.update_message(
            id.into_inner().into(),
            self.inner.jid.clone(),
            body,
            &self.inner.message_type,
        )
    }

    pub async fn toggle_reaction_to_message(&self, id: MessageId, emoji: Emoji) -> Result<()> {
        let message_id = message::Id::from(id.into_inner());
        let message = self.load_message(&message_id).await?;
        let mut emoji_found = false;

        let mut reactions = message
            .reactions
            .into_iter()
            .filter_map(|r| {
                if r.from.contains(&self.inner.user_jid) {
                    if r.emoji == emoji {
                        emoji_found = true;
                        return None;
                    }
                    Some(prose_xmpp::stanza::message::Emoji::from(
                        r.emoji.into_inner(),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if !emoji_found {
            reactions.push(prose_xmpp::stanza::message::Emoji::from(emoji.into_inner()))
        }

        let chat = self.inner.xmpp.get_mod::<mods::Chat>();
        chat.react_to_message(
            message_id,
            self.inner.jid.clone(),
            reactions,
            &self.inner.message_type,
        )?;

        Ok(())
    }

    pub async fn retract_message(&self, id: MessageId) -> Result<()> {
        let chat = self.inner.xmpp.get_mod::<mods::Chat>();
        chat.retract_message(
            id.into_inner().into(),
            self.inner.jid.clone(),
            &self.inner.message_type,
        )?;
        Ok(())
    }

    pub async fn load_messages_with_ids(&self, ids: &[MessageId]) -> Result<Vec<Message>> {
        let ids = ids
            .iter()
            .map(|id| id.as_ref().into())
            .collect::<Vec<message::Id>>();
        let messages = self
            .inner
            .client
            .data_cache
            .load_messages_targeting(&self.inner.jid, ids.as_slice(), None, true)
            .await?;
        Ok(Message::reducing_messages(messages))
    }

    pub async fn set_user_is_composing(&self, is_composing: bool) -> Result<()> {
        let chat = self.inner.xmpp.get_mod::<mods::Chat>();
        chat.send_chat_state(
            self.inner.jid.clone(),
            if is_composing {
                ChatState::Composing
            } else {
                ChatState::Paused
            },
            &self.inner.message_type,
        )
    }

    pub async fn load_composing_users(&self, conversation: &BareJid) -> Result<Vec<BareJid>> {
        // We currently do not support multi-user chats. So either our conversation partner is
        // typing or they are not.
        let conversation_partner_is_composing = self
            .inner
            .client
            .data_cache
            .load_chat_state(conversation)
            .await?
            == Some(ChatState::Composing);

        if conversation_partner_is_composing {
            Ok(vec![conversation.clone()])
        } else {
            Ok(vec![])
        }
    }

    pub async fn save_draft(&self, text: Option<&str>) -> Result<()> {
        self.inner
            .client
            .data_cache
            .save_draft(&self.inner.jid, text)
            .await?;
        Ok(())
    }

    pub async fn load_draft(&self) -> Result<Option<String>> {
        Ok(self
            .inner
            .client
            .data_cache
            .load_draft(&self.inner.jid)
            .await?)
    }
}
