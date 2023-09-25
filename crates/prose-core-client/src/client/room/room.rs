// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::avatar_cache::AvatarCache;
use crate::client::client::{ClientInner, ReceivedMessage};
use crate::data_cache::DataCache;
use crate::types::message_like::TimestampedMessage;
use crate::types::{ConnectedRoom, Message, MessageId, MessageLike};
use crate::{Client, ClientEvent};
use anyhow::{format_err, Result};
use chrono::{DateTime, Utc};
use jid::{BareJid, Jid};
use parking_lot::RwLock;
use prose_xmpp::stanza::message;
use prose_xmpp::stanza::message::{ChatState, Message as XMPPMessage};
use prose_xmpp::{mods, ns, Client as XMPPClient, TimeProvider};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;
use tracing::{debug, error, info};
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc;
use xmpp_parsers::muc::user::Affiliation;
use xmpp_parsers::muc::MucUser;
use xmpp_parsers::presence::Presence;

pub struct Group;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Occupant {
    /// The real JID of the occupant. Only available in non-anonymous rooms.
    pub jid: Option<BareJid>,
    pub affiliation: muc::user::Affiliation,
    pub occupant_id: Option<String>,
    pub chat_state: ChatState,
    pub chat_state_updated: DateTime<Utc>,
}

pub struct Room<Kind, D: DataCache + 'static, A: AvatarCache + 'static> {
    pub(super) inner: Arc<RoomInner<D, A>>,
    pub(super) inner_mut: Arc<RwLock<RoomInnerMut>>,
    /// Converts Room to a ConnectedRoom unless Room is pending.
    pub(super) to_connected_room:
        Arc<dyn Fn(Self) -> Result<ConnectedRoom<D, A>, ()> + Send + Sync>,
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
    /// The occupants of the room. The key is either the user's FullJid in a MUC room or the user's
    /// BareJid in direct message room.
    pub occupants: HashMap<Jid, Occupant>,
}

impl RoomInnerMut {
    pub(super) fn merge_members<'a>(&mut self, members: impl IntoIterator<Item = &'a Jid>) {
        for member in members.into_iter() {
            if !self.occupants.contains_key(member) {
                self.occupants.insert(
                    member.clone(),
                    Occupant {
                        jid: None,
                        affiliation: Affiliation::Member,
                        occupant_id: None,
                        chat_state: ChatState::Inactive,
                        chat_state_updated: Default::default(),
                    },
                );
            }
        }
    }
}

impl<Kind, D: DataCache, A: AvatarCache> Clone for Room<Kind, D, A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            inner_mut: self.inner_mut.clone(),
            to_connected_room: self.to_connected_room.clone(),
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
        let Some(from) = presence.from else {
            return Ok(());
        };

        info!(
            "Received presence from {} in room {}.",
            from, self.inner.jid
        );

        let Some(muc_user) = presence
            .payloads
            .into_iter()
            .filter_map(|payload| {
                if !payload.is("x", ns::MUC_USER) {
                    return None;
                }
                MucUser::try_from(payload).ok()
            })
            .take(1)
            .next()
        else {
            return Ok(());
        };

        // Let's try to pull out the real jid of our user…
        let Some(jid) = muc_user
            .items
            .into_iter()
            .filter_map(|item| item.jid)
            .take(1)
            .next()
        else {
            return Ok(());
        };

        info!("Received real jid for {}: {}", from, jid);

        {
            let mut inner_mut = self.inner_mut.write();
            let occupant = inner_mut.occupants.entry(from).or_default();
            occupant.jid = Some(jid.to_bare());
        }

        Ok(())
    }

    pub(super) async fn handle_received_message(&self, message: ReceivedMessage) -> Result<()> {
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

        struct ChatStateEvent {
            state: ChatState,
            from: Jid,
        }

        let mut chat_state: Option<ChatStateEvent> = None;

        if let ReceivedMessage::Message(message) = &message {
            if let (Some(state), Some(from)) = (&message.chat_state, &message.from) {
                chat_state = Some(ChatStateEvent {
                    state: state.clone(),
                    from: if message.r#type == MessageType::Groupchat {
                        from.clone()
                    } else {
                        Jid::Bare(from.to_bare())
                    },
                });
            }
        }

        let message_is_carbon = message.is_carbon();
        let now = self.inner.client.time_provider.now();

        let parsed_message: Result<MessageLike> = match message {
            ReceivedMessage::Message(message) => MessageLike::try_from(TimestampedMessage {
                message,
                timestamp: now.into(),
            }),
            ReceivedMessage::Carbon(carbon) => MessageLike::try_from(TimestampedMessage {
                message: carbon,
                timestamp: now.into(),
            }),
        };

        let parsed_message = match parsed_message {
            Ok(message) => Some(message),
            Err(err) => {
                error!("Failed to parse received message: {}", err);
                None
            }
        };

        if parsed_message.is_none() && chat_state.is_none() {
            // Nothing to do…
            return Ok(());
        }

        if let Some(message) = &parsed_message {
            debug!("Caching received message…");
            self.inner
                .client
                .data_cache
                .insert_messages([message])
                .await?;

            self.send_event(|room| ClientEvent::event_for_message(room, &message));
        }

        if let Some(chat_state) = chat_state {
            self.inner_mut
                .write()
                .occupants
                .entry(chat_state.from)
                .and_modify(|occupant| {
                    occupant.chat_state = chat_state.state;
                    occupant.chat_state_updated = now.clone().with_timezone(&Utc);
                });
            self.send_event(|room| ClientEvent::ComposingUsersChanged { room });
        }

        let Some(message) = parsed_message else {
            return Ok(());
        };

        // Don't send delivery receipts for carbons or anything other than a regular message.
        if message_is_carbon || !message.payload.is_message() {
            return Ok(());
        }

        if let Some(message_id) = message.id.into_original_id() {
            let chat = self.inner.xmpp.get_mod::<mods::Chat>();
            chat.mark_message_received(message_id, message.from, &self.inner.message_type)?;
        }

        Ok(())
    }

    pub(super) async fn handle_sent_message(&self, message: XMPPMessage) -> Result<()> {
        let timestamped_message = TimestampedMessage {
            message,
            timestamp: self.inner.client.time_provider.now(),
        };

        let message = MessageLike::try_from(timestamped_message)?;

        debug!("Caching sent message…");
        self.inner
            .client
            .data_cache
            .insert_messages([&message])
            .await?;

        self.send_event(|room| ClientEvent::event_for_message(room, &message));
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

impl<Kind, D: DataCache, A: AvatarCache> Room<Kind, D, A> {
    fn send_event(&self, builder: impl FnOnce(ConnectedRoom<D, A>) -> ClientEvent<D, A>) {
        let client = Client {
            client: self.inner.xmpp.clone(),
            inner: self.inner.client.clone(),
        };

        let room = match (self.to_connected_room)(self.clone()) {
            Ok(room) => room,
            Err(_) => {
                debug!("Not sending event from pending room.");
                return;
            }
        };

        client.send_event(builder(room))
    }
}
