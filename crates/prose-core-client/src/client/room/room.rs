// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::avatar_cache::AvatarCache;
use crate::client::client::{ClientInner, ReceivedMessage};
use crate::data_cache::DataCache;
use crate::types::{Message, MessageId};
use anyhow::{format_err, Result};
use jid::BareJid;
use parking_lot::RwLock;
use prose_xmpp::stanza::message;
use prose_xmpp::Client as XMPPClient;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc;
use xmpp_parsers::presence::Presence;

pub struct Group;
pub struct Generic;

#[derive(Debug, Clone, PartialEq)]
pub struct Occupant {
    pub affiliation: muc::user::Affiliation,
    pub occupant_id: Option<String>,
}

pub struct Room<Kind, D: DataCache + 'static, A: AvatarCache + 'static> {
    pub(super) inner: Arc<RoomInner<D, A>>,
    pub(super) inner_mut: Arc<RwLock<RoomInnerMut>>,
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

    pub xmpp: XMPPClient,
    pub client: Arc<ClientInner<D, A>>,
    pub message_type: MessageType,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub(super) struct RoomInnerMut {
    /// The room's subject.
    pub subject: Option<String>,
    /// The occupants of the room.
    pub occupants: Vec<Occupant>,
}

impl<Kind, D: DataCache, A: AvatarCache> Clone for Room<Kind, D, A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            inner_mut: self.inner_mut.clone(),
            _type: Default::default(),
        }
    }
}

impl<Kind, D: DataCache, A: AvatarCache> Debug for Room<Kind, D, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Room")
            .field("jid", &self.inner.jid)
            .field("name", &self.inner.name)
            .field("description", &self.inner.description)
            .field("user_jid", &self.inner.user_jid)
            .field("user_nickname", &self.inner.user_nickname)
            .field("subject", &self.inner_mut.read().subject)
            .field("occupants", &self.inner_mut.read().occupants)
            .finish_non_exhaustive()
    }
}

impl<Kind, D: DataCache, A: AvatarCache> PartialEq for Room<Kind, D, A> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.jid == other.inner.jid
    }
}

impl<Kind, D: DataCache, A: AvatarCache> Room<Kind, D, A> {
    pub(super) async fn handle_presence(&self, presence: Presence) -> Result<()> {
        //println!("RECEIVED PRESENCE: {:?}", presence);
        Ok(())
    }

    pub(super) async fn handle_message(&self, message: ReceivedMessage) -> Result<()> {
        if let ReceivedMessage::Message(message) = &message {
            if let Some(subject) = &message.subject {
                self.inner_mut.write().subject = if subject.is_empty() {
                    None
                } else {
                    Some(subject.to_string())
                };
                return Ok(());
            }
        }

        Ok(())
    }

    pub(super) async fn load_message(&self, message_id: &message::Id) -> Result<Message> {
        let ids = [MessageId::from(message_id.as_ref())];
        self.load_messages_with_ids(&ids)
            .await?
            .pop()
            .ok_or(format_err!("No message with id {}", ids[0]))
    }
}
