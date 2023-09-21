// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use anyhow::{bail, Result};
use jid::{BareJid, FullJid};
use parking_lot::lock_api::RwLock;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::presence::Presence;

use super::Room;
use crate::avatar_cache::AvatarCache;
use crate::client::client::{ClientInner, ReceivedMessage};
use crate::client::room;
use crate::data_cache::DataCache;
use crate::room::room::{RoomInner, RoomInnerMut};
use crate::types::muc::RoomMetadata;
use crate::types::Contact;
use crate::Client;
use prose_xmpp::Client as XMPPClient;

pub(in crate::client) enum RoomEnvelope<D: DataCache + 'static, A: AvatarCache + 'static> {
    /// A room that we're in the process of joining
    Pending(Room<room::Base, D, A>),
    DirectMessage(Room<room::DirectMessage, D, A>),
    Group(Room<room::Group, D, A>),
    PrivateChannel(Room<room::PrivateChannel, D, A>),
    PublicChannel(Room<room::PublicChannel, D, A>),
    /// A generic MUC room that doesn't match any of our requirements
    Generic(Room<room::Generic, D, A>),
}

macro_rules! unwrap_room {
    ($envelope:expr, $method_call:ident($( $arg:expr ),*) .await) => {
        match $envelope {
            RoomEnvelope::Pending(room) => room.$method_call($($arg),*).await,
            RoomEnvelope::DirectMessage(room) => room.$method_call($($arg),*).await,
            RoomEnvelope::Group(room) => room.$method_call($($arg),*).await,
            RoomEnvelope::PrivateChannel(room) => room.$method_call($($arg),*).await,
            RoomEnvelope::PublicChannel(room) => room.$method_call($($arg),*).await,
            RoomEnvelope::Generic(room) => room.$method_call($($arg),*).await,
        }
    };
    ($envelope:expr, $method_call:ident($( $arg:expr ),*)) => {
        match $envelope {
            RoomEnvelope::Pending(room) => room.$method_call($($arg),*),
            RoomEnvelope::DirectMessage(room) => room.$method_call($($arg),*),
            RoomEnvelope::Group(room) => room.$method_call($($arg),*),
            RoomEnvelope::PrivateChannel(room) => room.$method_call($($arg),*),
            RoomEnvelope::PublicChannel(room) => room.$method_call($($arg),*),
            RoomEnvelope::Generic(room) => room.$method_call($($arg),*),
        }
    };
}

impl<D: DataCache, A: AvatarCache> Debug for RoomEnvelope<D, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending(room) => write!(f, "RoomEnvelope::Pending({:?}", room),
            Self::DirectMessage(room) => write!(f, "RoomEnvelope::DirectMessage({:?}", room),
            Self::Group(room) => write!(f, "RoomEnvelope::Group({:?}", room),
            Self::PrivateChannel(room) => write!(f, "RoomEnvelope::PrivateChannel({:?}", room),
            Self::PublicChannel(room) => write!(f, "RoomEnvelope::PublicChannel({:?}", room),
            Self::Generic(room) => write!(f, "RoomEnvelope::Generic({:?}", room),
        }
    }
}

impl<D: DataCache, A: AvatarCache> Clone for RoomEnvelope<D, A> {
    fn clone(&self) -> Self {
        match self {
            Self::Pending(room) => Self::Pending(room.clone()),
            Self::DirectMessage(room) => Self::DirectMessage(room.clone()),
            Self::Group(room) => Self::Group(room.clone()),
            Self::PrivateChannel(room) => Self::PrivateChannel(room.clone()),
            Self::PublicChannel(room) => Self::PublicChannel(room.clone()),
            Self::Generic(room) => Self::Generic(room.clone()),
        }
    }
}

impl<D: DataCache, A: AvatarCache> RoomEnvelope<D, A> {
    pub fn to_base_room(&self) -> Room<room::Base, D, A> {
        match self {
            Self::Pending(room) => room.to_base(),
            Self::DirectMessage(room) => room.to_base(),
            Self::Group(room) => room.to_base(),
            Self::PrivateChannel(room) => room.to_base(),
            Self::PublicChannel(room) => room.to_base(),
            Self::Generic(room) => room.to_base(),
        }
    }
}

impl<D: DataCache, A: AvatarCache> RoomEnvelope<D, A> {
    pub fn pending(
        room_jid: &BareJid,
        user_jid: &BareJid,
        nickname: &str,
        client: &Client<D, A>,
    ) -> Self {
        Self::Pending(Room {
            inner: Arc::new(RoomInner {
                jid: room_jid.clone(),
                name: None,
                description: None,
                user_jid: user_jid.clone(),
                user_nickname: nickname.to_string(),
                xmpp: client.client.clone(),
                client: client.inner.clone(),
                message_type: Default::default(),
            }),
            inner_mut: Default::default(),
            _type: Default::default(),
        })
    }

    pub fn jid(&self) -> &BareJid {
        unwrap_room!(self, jid())
    }

    pub fn user_nickname(&self) -> &str {
        unwrap_room!(self, user_nickname())
    }

    pub async fn handle_presence(&self, presence: Presence) -> Result<()> {
        unwrap_room!(self, handle_presence(presence).await)
    }

    pub async fn handle_message(&self, message: ReceivedMessage) -> Result<()> {
        unwrap_room!(self, handle_message(message).await)
    }

    pub fn promote_to_permanent_room(self, metadata: RoomMetadata) -> Result<Self> {
        let Self::Pending(pending_room) = self else {
            bail!("Cannot promote non-pending room");
        };

        let inner_mut = pending_room.inner_mut.read().clone();

        Ok(Self::from((
            metadata,
            pending_room.inner.user_jid.clone(),
            pending_room.inner.xmpp.clone(),
            pending_room.inner.client.clone(),
            inner_mut,
        )))
    }
}

impl<D: DataCache, A: AvatarCache> From<(RoomMetadata, BareJid, &Client<D, A>)>
    for RoomEnvelope<D, A>
{
    fn from(value: (RoomMetadata, BareJid, &Client<D, A>)) -> Self {
        (
            value.0,
            value.1,
            value.2.client.clone(),
            value.2.inner.clone(),
            Default::default(),
        )
            .into()
    }
}

impl<D: DataCache, A: AvatarCache>
    From<(
        RoomMetadata,
        BareJid,
        XMPPClient,
        Arc<ClientInner<D, A>>,
        RoomInnerMut,
    )> for RoomEnvelope<D, A>
{
    fn from(
        value: (
            RoomMetadata,
            BareJid,
            XMPPClient,
            Arc<ClientInner<D, A>>,
            RoomInnerMut,
        ),
    ) -> Self {
        fn make_room<Kind, D: DataCache, A: AvatarCache>(
            value: (
                RoomMetadata,
                BareJid,
                XMPPClient,
                Arc<ClientInner<D, A>>,
                RoomInnerMut,
            ),
            message_type: MessageType,
        ) -> Room<Kind, D, A> {
            let (metadata, user_jid, xmpp, client, inner_mut) = value;

            Room {
                inner: Arc::new(RoomInner {
                    jid: metadata.room_jid.to_bare(),
                    user_nickname: metadata.room_jid.resource_str().to_string(),
                    name: metadata.settings.name,
                    description: metadata.settings.description,
                    user_jid,
                    xmpp,
                    client,
                    message_type,
                }),
                inner_mut: Arc::new(RwLock::new(inner_mut)),
                _type: Default::default(),
            }
        }

        let features = &value.0.settings.features;

        match features {
            _ if features.can_act_as_group() => {
                Self::Group(make_room(value, MessageType::Groupchat))
            }
            _ if features.can_act_as_private_channel() => {
                Self::PrivateChannel(make_room(value, MessageType::Groupchat))
            }
            _ if features.can_act_as_public_channel() => {
                Self::PublicChannel(make_room(value, MessageType::Groupchat))
            }
            _ => Self::Generic(make_room(value, MessageType::Groupchat)),
        }
    }
}

impl<D: DataCache, A: AvatarCache> From<(Contact, FullJid, &Client<D, A>)> for RoomEnvelope<D, A> {
    fn from(value: (Contact, FullJid, &Client<D, A>)) -> Self {
        let (contact, user_jid, client) = value;

        let room = Room {
            inner: Arc::new(RoomInner {
                jid: contact.jid,
                name: Some(contact.name),
                description: None,
                user_jid: user_jid.to_bare(),
                user_nickname: user_jid.resource_str().to_string(),
                xmpp: client.client.clone(),
                client: client.inner.clone(),
                message_type: MessageType::Chat,
            }),
            inner_mut: Default::default(),
            _type: Default::default(),
        };

        Self::DirectMessage(room)
    }
}