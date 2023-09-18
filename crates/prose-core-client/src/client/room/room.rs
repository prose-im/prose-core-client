// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::avatar_cache::AvatarCache;
use crate::client::client::ClientInner;
use crate::data_cache::DataCache;
use crate::types::{Message, MessageId, MessageLike};
use anyhow::{format_err, Result};
use jid::BareJid;
use prose_xmpp::mods;
use prose_xmpp::stanza::message;
use prose_xmpp::stanza::message::{ChatState, Emoji};
use prose_xmpp::Client as XMPPClient;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::debug;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc;

pub struct Group;
pub struct Generic;

#[derive(Debug, Clone, PartialEq)]
pub struct Occupant {
    pub affiliation: muc::user::Affiliation,
    pub occupant_id: Option<String>,
}

pub struct Room<Kind, D: DataCache + 'static, A: AvatarCache + 'static> {
    pub(super) inner: Arc<RoomInner<D, A>>,
    pub(super) _type: PhantomData<Kind>,
}

pub(super) struct RoomInner<D: DataCache + 'static, A: AvatarCache + 'static> {
    /// The JID of the room.
    pub jid: BareJid,
    /// The name of the room.
    pub name: Option<String>,
    /// The description of the room.
    pub description: Option<String>,
    /// The JID of our logged-in user.
    pub user_jid: BareJid,
    /// The nickname with which our user is connected to the room.
    pub user_nickname: String,
    /// The occupants of the room.
    pub occupants: Vec<Occupant>,

    pub xmpp: XMPPClient,
    pub client: Arc<ClientInner<D, A>>,
    pub message_type: MessageType,
}

impl<Kind, D: DataCache, A: AvatarCache> Clone for Room<Kind, D, A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _type: Default::default(),
        }
    }
}

impl<Kind, D: DataCache, A: AvatarCache> PartialEq for Room<Kind, D, A> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.jid == other.inner.jid
    }
}

const MESSAGE_PAGE_SIZE: u32 = 50;

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
}

impl<Kind, D: DataCache, A: AvatarCache> Room<Kind, D, A> {
    pub async fn send_message(&self, body: impl Into<String>) -> Result<()> {
        let chat = self.inner.xmpp.get_mod::<mods::Chat>();
        chat.send_message(
            self.inner.jid.clone(),
            body,
            self.inner.message_type.clone(),
            Some(ChatState::Active),
        )
    }

    pub async fn load_latest_messages(
        &self,
        _since: impl Into<Option<&MessageId>> + Debug,
        _load_from_server: bool,
    ) -> Result<Vec<Message>> {
        debug!("Loading messages from server…");

        let mam = self.inner.xmpp.get_mod::<mods::MAM>();
        let result = mam
            .load_messages_in_chat(
                &self.inner.jid,
                None,
                None,
                Some(MESSAGE_PAGE_SIZE as usize),
            )
            .await?;

        let messages = result
            .0
            .iter()
            .map(|msg| MessageLike::try_from(msg))
            .collect::<Result<Vec<_>, _>>()?;

        debug!("Found {} messages. Saving to cache…", messages.len());
        self.inner
            .client
            .data_cache
            .insert_messages(messages.iter())
            .await?;

        Ok(Message::reducing_messages(messages))
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

    pub async fn update_message(&self, id: MessageId, body: impl Into<String>) -> Result<()> {
        let chat = self.inner.xmpp.get_mod::<mods::Chat>();
        chat.update_message(id.into_inner().into(), self.inner.jid.clone(), body)
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
        chat.react_to_message(message_id, self.inner.jid.clone(), reactions)?;

        Ok(())
    }

    pub async fn retract_message(&self, id: MessageId) -> Result<()> {
        let chat = self.inner.xmpp.get_mod::<mods::Chat>();
        chat.retract_message(id.into_inner().into(), self.inner.jid.clone())?;
        Ok(())
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

impl<Kind, D: DataCache, A: AvatarCache> Room<Kind, D, A> {
    pub(super) async fn load_message(&self, message_id: &message::Id) -> Result<Message> {
        let ids = [MessageId::from(message_id.as_ref())];
        self.load_messages_with_ids(&ids)
            .await?
            .pop()
            .ok_or(format_err!("No message with id {}", ids[0]))
    }
}
