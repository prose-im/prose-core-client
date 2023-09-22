// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::Utc;
use jid::{BareJid, Jid};
use std::sync::atomic::Ordering;
use tracing::{debug, error};
use xmpp_parsers::presence::Presence;

use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::mods::{bookmark, bookmark2, caps, chat, muc, ping, profile, status};

use prose_xmpp::stanza::{avatar, Message, UserActivity, VCard4};
use prose_xmpp::{client, mods, Event, TimeProvider};

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::message_like::{Payload, TimestampedMessage};
use crate::types::{AvatarMetadata, ConnectedRoom, MessageLike, UserProfile};
use crate::{types, CachePolicy, Client, ClientEvent, ConnectionEvent};

#[allow(dead_code)]
enum Request {
    Ping {
        from: Jid,
        id: String,
    },
    DiscoInfo {
        from: Jid,
        id: String,
        node: Option<String>,
    },
    EntityTime {
        from: Jid,
        id: String,
    },
    SoftwareVersion {
        from: Jid,
        id: String,
    },
    LastActivity {
        from: Jid,
        id: String,
    },
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub(super) async fn handle_event(&self, event: Event) {
        let result = match event {
            Event::Client(event) => match event {
                client::Event::Connected => {
                    self.inner
                        .is_observing_rooms
                        .store(false, Ordering::Relaxed);
                    self.send_event(ClientEvent::ConnectionStatusChanged {
                        event: ConnectionEvent::Connect,
                    });
                    Ok(())
                }
                client::Event::Disconnected { error } => {
                    self.send_event(ClientEvent::ConnectionStatusChanged {
                        event: ConnectionEvent::Disconnect { error },
                    });
                    Ok(())
                }
            },

            Event::Caps(event) => match event {
                caps::Event::DiscoInfoQuery { from, id, node } => {
                    self.handle_request(Request::DiscoInfo { from, id, node })
                        .await
                }
                caps::Event::Caps { .. } => Ok(()),
            },

            Event::Chat(event) => match event {
                chat::Event::Message(message) => {
                    self.did_receive_message(ReceivedMessage::Message(message))
                        .await
                }
                chat::Event::Carbon(carbon) => {
                    self.did_receive_message(ReceivedMessage::Carbon(carbon))
                        .await
                }
                chat::Event::Sent(message) => self.did_send_message(message).await,
            },

            Event::Ping(event) => match event {
                ping::Event::Ping { from, id } => {
                    self.handle_request(Request::Ping { from, id }).await
                }
            },

            Event::Profile(event) => match event {
                profile::Event::Vcard { from, vcard } => self.vcard_did_change(from, vcard).await,
                profile::Event::AvatarMetadata { from, metadata } => {
                    self.avatar_metadata_did_change(from, metadata).await
                }
                profile::Event::EntityTimeQuery { from, id } => {
                    self.handle_request(Request::EntityTime { from, id }).await
                }
                profile::Event::SoftwareVersionQuery { from, id } => {
                    self.handle_request(Request::SoftwareVersion { from, id })
                        .await
                }
                profile::Event::LastActivityQuery { from, id } => {
                    self.handle_request(Request::LastActivity { from, id })
                        .await
                }
            },

            Event::Status(event) => match event {
                status::Event::Presence(presence) => self.presence_did_change(presence).await,
                status::Event::UserActivity {
                    from,
                    user_activity,
                } => self.user_activity_did_change(from, user_activity).await,
            },

            Event::Bookmark(event) => match event {
                bookmark::Event::BookmarksChanged { bookmarks } => {
                    self.handle_changed_bookmarks(bookmarks).await
                }
            },

            Event::Bookmark2(event) => match event {
                bookmark2::Event::BookmarksPublished { bookmarks } => {
                    self.handle_published_bookmarks2(bookmarks).await
                }
                bookmark2::Event::BookmarksRetracted { jids } => {
                    self.handle_retracted_bookmarks2(jids).await
                }
            },

            Event::MUC(event) => match event {
                muc::Event::DirectInvite { from, invite } => {
                    self.handle_direct_invite(from, invite).await
                }
                muc::Event::MediatedInvite { from, invite } => {
                    self.handle_mediated_invite(from, invite).await
                }
            },
        };

        if let Err(err) = result {
            error!("Failed to handle event. {}", err)
        }
    }
}

#[derive(Debug)]
pub(in crate::client) enum ReceivedMessage {
    Message(Message),
    Carbon(Carbon),
}

impl ReceivedMessage {
    pub fn is_carbon(&self) -> bool {
        match self {
            Self::Message(_) => false,
            Self::Carbon(_) => true,
        }
    }

    pub fn from(&self) -> Option<BareJid> {
        match &self {
            ReceivedMessage::Message(message) => message.from.as_ref().map(|jid| jid.to_bare()),
            ReceivedMessage::Carbon(Carbon::Received(message)) => message
                .stanza
                .as_ref()
                .and_then(|message| message.from.as_ref())
                .map(|jid| jid.to_bare()),
            ReceivedMessage::Carbon(Carbon::Sent(message)) => message
                .stanza
                .as_ref()
                .and_then(|message| message.from.as_ref())
                .map(|jid| jid.to_bare()),
        }
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub(in crate::client) fn send_event(&self, event: ClientEvent<D, A>) {
        let Some(delegate) = &self.inner.delegate else {
            return;
        };
        let client = Client {
            client: self.client.clone(),
            inner: self.inner.clone(),
        };
        delegate.handle_event(client, event);
    }

    pub(in crate::client) fn send_event_for_message(
        &self,
        room: ConnectedRoom<D, A>,
        message: &MessageLike,
    ) {
        if self.inner.delegate.is_none() {
            return;
        }

        let event = if let Some(ref target) = message.target {
            if message.payload == Payload::Retraction {
                ClientEvent::MessagesDeleted {
                    room,
                    message_ids: vec![target.as_ref().into()],
                }
            } else {
                ClientEvent::MessagesUpdated {
                    room,
                    message_ids: vec![target.as_ref().into()],
                }
            }
        } else {
            ClientEvent::MessagesAppended {
                room,
                message_ids: vec![message.id.id().as_ref().into()],
            }
        };
        self.send_event(event)
    }

    async fn vcard_did_change(&self, from: Jid, vcard: VCard4) -> Result<()> {
        debug!("New vcard for {} {:?}", from, vcard);

        let Some(profile): Option<UserProfile> = vcard.clone().try_into().ok() else {
            return Ok(());
        };

        let from = from.into_bare();

        self.inner
            .data_cache
            .insert_user_profile(&from, &profile)
            .await?;
        self.send_event(ClientEvent::ContactChanged { jid: from });

        Ok(())
    }

    async fn avatar_metadata_did_change(
        &self,
        from: Jid,
        metadata: avatar::Metadata,
    ) -> Result<()> {
        debug!("New metadata for {} {:?}", from, metadata);

        let Some(metadata) = metadata
            .infos
            .first()
            .map(|i| AvatarMetadata::from(i.clone()))
        else {
            return Ok(());
        };

        let from = from.into_bare();

        self.inner
            .data_cache
            .insert_avatar_metadata(&from, &metadata)
            .await?;

        self.load_and_cache_avatar_image(&from, &metadata, CachePolicy::ReturnCacheDataElseLoad)
            .await?;

        self.send_event(ClientEvent::AvatarChanged { jid: from });

        Ok(())
    }

    async fn presence_did_change(&self, presence: Presence) -> Result<()> {
        let Some(from) = presence.from.clone() else {
            return Ok(());
        };

        let jid = from.to_bare();

        // If the presence was sent from a JID that belongs to one of our connected rooms, let
        // that room handle it…
        let room = self.inner.connected_rooms.read().get(&jid).cloned();
        if let Some(room) = room {
            room.handle_presence(presence).await?;
            return Ok(());
        };

        // Update user presences with the received one and retrieve the new highest presence…
        let highest_presence = self.update_presence(&from, presence.into());

        // …update the cache…
        self.inner
            .data_cache
            .insert_presence(&jid, &highest_presence)
            .await?;

        // …and finally let our delegate know.
        self.send_event(ClientEvent::ContactChanged { jid });
        Ok(())
    }

    async fn user_activity_did_change(&self, from: Jid, user_activity: UserActivity) -> Result<()> {
        let jid = from.into_bare();

        let user_activity = types::UserActivity::try_from(user_activity).ok();

        self.inner
            .data_cache
            .insert_user_activity(&jid, &user_activity)
            .await?;
        self.send_event(ClientEvent::ContactChanged { jid });

        Ok(())
    }

    async fn handle_request(&self, request: Request) -> Result<()> {
        match request {
            Request::Ping { from, id } => {
                let ping = self.client.get_mod::<mods::Ping>();
                ping.send_pong(from, id).await?
            }
            Request::DiscoInfo { from, id, node: _ } => {
                let caps = self.client.get_mod::<mods::Caps>();
                caps.send_disco_info_query_response(from, id, (&self.inner.caps).into())
                    .await?
            }
            Request::EntityTime { from, id } => {
                let profile = self.client.get_mod::<mods::Profile>();
                profile
                    .send_entity_time_response(self.inner.time_provider.now(), from, id)
                    .await?
            }
            Request::SoftwareVersion { from, id } => {
                let profile = self.client.get_mod::<mods::Profile>();
                profile
                    .send_software_version_response(
                        self.inner.software_version.clone().into(),
                        from,
                        id,
                    )
                    .await?
            }
            Request::LastActivity { from, id } => {
                let profile = self.client.get_mod::<mods::Profile>();
                profile
                    .send_last_activity_response(0, None, from, id)
                    .await?
            }
        }
        Ok(())
    }

    async fn did_receive_message(&self, message: ReceivedMessage) -> Result<()> {
        let Some(from) = message.from() else {
            error!("Received message without 'from'");
            return Ok(());
        };

        let Some(room) = self.inner.connected_rooms.read().get(&from).cloned() else {
            todo!("Received message from sender for which we do not have a room.");
        };

        room.handle_message(message).await?;
        return Ok(());
    }

    async fn did_send_message(&self, message: Message) -> Result<()> {
        // TODO: Inject date from outside for testing
        let timestamped_message = TimestampedMessage {
            message,
            timestamp: Utc::now().into(),
        };

        let message = MessageLike::try_from(timestamped_message)?;

        debug!("Caching sent message…");
        self.inner.data_cache.insert_messages([&message]).await?;
        // self.send_event_for_message(&message.to, &message);
        // todo!("FIXME");

        Ok(())
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    /// Updates the presences for `from` and returns the new highest presence.
    fn update_presence(&self, from: &Jid, presence: types::Presence) -> types::Presence {
        let mut map = self.inner.presences.write();
        map.update_presence(&from, presence.into());
        map.get_highest_presence(&from.to_bare())
            .map(|entry| entry.presence.clone())
            .unwrap_or(types::Presence::unavailable())
    }
}
