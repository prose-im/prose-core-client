// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::Utc;
use jid::{BareJid, Jid};
use tracing::{debug, error};
use xmpp_parsers::presence::Presence;

use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::mods::{caps, chat, profile, status};
use prose_xmpp::stanza::message::ChatState;
use prose_xmpp::stanza::{avatar, Message, UserActivity, VCard4};
use prose_xmpp::{client, mods, Event};

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::message_like::{Payload, TimestampedMessage};
use crate::types::{AvatarMetadata, MessageLike, UserProfile};
use crate::{types, CachePolicy, Client, ClientEvent, ConnectionEvent};

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub(super) async fn handle_event(&self, event: Event) {
        let result = match event {
            Event::Client(event) => match event {
                client::Event::Connected => {
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
                    self.did_receive_disco_info_query(from, id, node).await
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

            Event::Profile(event) => match event {
                profile::Event::Vcard { from, vcard } => self.vcard_did_change(from, vcard).await,
                profile::Event::AvatarMetadata { from, metadata } => {
                    self.avatar_metadata_did_change(from, metadata).await
                }
            },

            Event::Status(event) => match event {
                status::Event::Presence(presence) => self.presence_did_change(presence).await,
                status::Event::UserActivity {
                    from,
                    user_activity,
                } => self.user_activity_did_change(from, user_activity).await,
            },
        };

        if let Err(err) = result {
            error!("Failed to handle event. {}", err)
        }
    }
}

#[derive(Debug)]
pub enum ReceivedMessage {
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
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    fn send_event(&self, event: ClientEvent) {
        let Some(delegate) = &self.inner.delegate else {
            return;
        };
        let client = Client {
            client: self.client.clone(),
            inner: self.inner.clone(),
        };
        delegate.handle_event(client, event);
    }

    fn send_event_for_message(&self, conversation: &BareJid, message: &MessageLike) {
        if self.inner.delegate.is_none() {
            return;
        }

        let event = if let Some(ref target) = message.target {
            if message.payload == Payload::Retraction {
                ClientEvent::MessagesDeleted {
                    conversation: conversation.clone(),
                    message_ids: vec![target.as_ref().into()],
                }
            } else {
                ClientEvent::MessagesUpdated {
                    conversation: conversation.clone(),
                    message_ids: vec![target.as_ref().into()],
                }
            }
        } else {
            ClientEvent::MessagesAppended {
                conversation: conversation.clone(),
                message_ids: vec![message.id.as_ref().into()],
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

        let jid = from.into_bare();

        self.inner
            .data_cache
            .insert_presence(&jid, &types::Presence::from(presence))
            .await?;

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

    async fn did_receive_disco_info_query(
        &self,
        from: Jid,
        id: String,
        _node: String,
    ) -> Result<()> {
        let caps = self.client.get_mod::<mods::Caps>();
        caps.send_disco_info_query_response(from, id, (&self.inner.caps).into())
            .await
    }

    async fn did_receive_message(&self, message: ReceivedMessage) -> Result<()> {
        struct ChatStateEvent {
            state: ChatState,
            from: BareJid,
        }

        let mut chat_state: Option<ChatStateEvent> = None;

        if let ReceivedMessage::Message(message) = &message {
            if let (Some(state), Some(from)) = (&message.chat_state, &message.from) {
                chat_state = Some(ChatStateEvent {
                    state: state.clone(),
                    from: from.to_bare(),
                });
            }
        }

        let message_is_carbon = message.is_carbon();
        let now = Utc::now();

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
            self.inner.data_cache.insert_messages([message]).await?;

            let conversation = if message_is_carbon {
                &message.to
            } else {
                &message.from
            };
            self.send_event_for_message(conversation, message);
        }

        if let Some(chat_state) = chat_state {
            self.inner
                .data_cache
                .insert_chat_state(&chat_state.from, &chat_state.state)
                .await?;
            self.send_event(ClientEvent::ComposingUsersChanged {
                conversation: chat_state.from,
            })
        }

        let Some(message) = parsed_message else {
            return Ok(());
        };

        // Don't send delivery receipts for carbons or anything other than a regular message.
        if message_is_carbon || !message.payload.is_message() {
            return Ok(());
        }

        let chat = self.client.get_mod::<mods::Chat>();
        chat.mark_message_received(message.id.clone(), message.from)?;

        Ok(())
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
        self.send_event_for_message(&message.to, &message);

        Ok(())
    }
}
