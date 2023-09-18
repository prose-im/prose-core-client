// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp::Ordering;
use std::sync::Arc;

use jid::BareJid;
use prose_xmpp::RequestError;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::presence::Presence;

use crate::avatar_cache::AvatarCache;
use crate::client::room;
use crate::client::room::Room;
use crate::data_cache::DataCache;
use crate::room::room::RoomInner;
use crate::types::muc::RoomMetadata;
use crate::Client;

pub enum RoomEnvelope<D: DataCache + 'static, A: AvatarCache + 'static> {
    DirectMessage(Room<room::DirectMessage, D, A>),
    Group(Room<room::Group, D, A>),
    PrivateChannel(Room<room::PrivateChannel, D, A>),
    PublicChannel(Room<room::PublicChannel, D, A>),
    /// A generic MUC room that doesn't match any of our requirements
    Generic(Room<room::Generic, D, A>),
}

macro_rules! unwrap_room {
    ($envelope:expr, $accessor:ident) => {
        match $envelope {
            Self::DirectMessage(room) => room.$accessor(),
            Self::Group(room) => room.$accessor(),
            Self::PrivateChannel(room) => room.$accessor(),
            Self::PublicChannel(room) => room.$accessor(),
            Self::Generic(room) => room.$accessor(),
        }
    };
}

impl<D: DataCache, A: AvatarCache> RoomEnvelope<D, A> {
    pub fn jid(&self) -> &BareJid {
        unwrap_room!(self, jid)
    }

    pub fn name(&self) -> Option<&str> {
        unwrap_room!(self, name)
    }

    pub fn user_nickname(&self) -> &str {
        unwrap_room!(self, user_nickname)
    }
}

impl<D: DataCache, A: AvatarCache> RoomEnvelope<D, A> {
    pub(crate) fn handle_presence(&mut self, presence: Presence) {
        println!("RECEIVED PRESENCE: {:?}", presence);
    }
}

impl<D: DataCache, A: AvatarCache> RoomEnvelope<D, A> {
    fn sort_value(&self) -> i32 {
        match self {
            Self::DirectMessage(_) => 0,
            Self::Group(_) => 0,
            Self::PrivateChannel(_) => 1,
            Self::PublicChannel(_) => 2,
            Self::Generic(_) => 3,
        }
    }
}

impl<D: DataCache, A: AvatarCache> Clone for RoomEnvelope<D, A> {
    fn clone(&self) -> Self {
        match self {
            Self::DirectMessage(room) => Self::DirectMessage(room.clone()),
            Self::Group(room) => Self::Group(room.clone()),
            Self::PrivateChannel(room) => Self::PrivateChannel(room.clone()),
            Self::PublicChannel(room) => Self::PublicChannel(room.clone()),
            Self::Generic(room) => Self::Generic(room.clone()),
        }
    }
}

impl<D: DataCache, A: AvatarCache> Eq for RoomEnvelope<D, A> {}

impl<D: DataCache, A: AvatarCache> PartialEq for RoomEnvelope<D, A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::DirectMessage(lhs), Self::DirectMessage(rhs)) => lhs == rhs,
            (Self::Group(lhs), Self::Group(rhs)) => lhs == rhs,
            (Self::PrivateChannel(lhs), Self::PrivateChannel(rhs)) => lhs == rhs,
            (Self::PublicChannel(lhs), Self::PublicChannel(rhs)) => lhs == rhs,
            (Self::Generic(lhs), Self::Generic(rhs)) => lhs == rhs,
            (Self::DirectMessage(_), _)
            | (Self::Group(_), _)
            | (Self::PrivateChannel(_), _)
            | (Self::PublicChannel(_), _)
            | (Self::Generic(_), _) => false,
        }
    }
}

impl<D: DataCache, A: AvatarCache> PartialOrd for RoomEnvelope<D, A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<D: DataCache, A: AvatarCache> Ord for RoomEnvelope<D, A> {
    fn cmp(&self, other: &Self) -> Ordering {
        let sort_val1 = self.sort_value();
        let sort_val2 = other.sort_value();

        if sort_val1 < sort_val2 {
            return Ordering::Less;
        } else if sort_val1 > sort_val2 {
            return Ordering::Greater;
        }

        self.name()
            .unwrap_or_default()
            .cmp(other.name().unwrap_or_default())
    }
}

impl<D: DataCache, A: AvatarCache> TryFrom<(RoomMetadata, &Client<D, A>)> for RoomEnvelope<D, A> {
    type Error = RequestError;

    fn try_from(value: (RoomMetadata, &Client<D, A>)) -> Result<Self, Self::Error> {
        fn make_room<Kind, D: DataCache, A: AvatarCache>(
            value: (RoomMetadata, &Client<D, A>),
            message_type: MessageType,
        ) -> Result<Room<Kind, D, A>, RequestError> {
            Ok(Room {
                inner: Arc::new(RoomInner {
                    jid: value.0.room_jid.to_bare(),
                    user_nickname: value.0.room_jid.resource_str().to_string(),
                    name: value.0.settings.name,
                    description: value.0.settings.description,
                    user_jid: value
                        .1
                        .connected_jid()
                        .map_err(|err| RequestError::Generic {
                            msg: err.to_string(),
                        })?
                        .into_bare(),
                    xmpp: value.1.client.clone(),
                    client: value.1.inner.clone(),
                    occupants: vec![],
                    message_type,
                }),
                _type: Default::default(),
            })
        }

        let features = &value.0.settings.features;

        Ok(match features {
            _ if features.can_act_as_group() => {
                Self::Group(make_room(value, MessageType::Groupchat)?)
            }
            _ if features.can_act_as_private_channel() => {
                Self::PrivateChannel(make_room(value, MessageType::Groupchat)?)
            }
            _ if features.can_act_as_public_channel() => {
                Self::PublicChannel(make_room(value, MessageType::Groupchat)?)
            }
            _ => Self::Generic(make_room(value, MessageType::Groupchat)?),
        })
    }
}
