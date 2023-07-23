use anyhow::Result;
use chrono::Utc;
use jid::{BareJid, Jid};
use tracing::{debug, error};
use xmpp_parsers::presence;
use xmpp_parsers::presence::Presence;

use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::message::ChatState;
use prose_xmpp::stanza::{avatar, Message, VCard4};
use prose_xmpp::{mods, Event};

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::domain_ext::UserProfile;
use crate::types::message_like::{Payload, TimestampedMessage};
use crate::types::{AvatarMetadata, MessageLike};
use crate::{types, Client, ClientEvent};

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub(super) async fn handle_event(&self, event: Event) {
        let result = match event {
            Event::Connected => Ok(()),
            Event::Disconnected { .. } => Ok(()),
            Event::DiscoInfoQuery { from, id, node } => {
                self.did_receive_disco_info_query(from, id, node).await
            }
            Event::CapsPresence { .. } => Ok(()),
            Event::Message(message) => {
                self.did_receive_message(ReceivedMessage::Message(message))
                    .await
            }
            Event::Carbon(carbon) => {
                self.did_receive_message(ReceivedMessage::Carbon(carbon))
                    .await
            }
            Event::Sent(message) => self.did_send_message(message).await,
            Event::Vcard { from, vcard } => self.vcard_did_change(from, vcard).await,
            Event::AvatarMetadata { from, metadata } => {
                self.avatar_metadata_did_change(from, metadata).await
            }
            Event::Presence(presence) => self.presence_did_change(presence).await,
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
        delegate.handle_event(event);
    }

    fn send_event_for_message(&self, conversation: &BareJid, message: &MessageLike) {
        let Some(delegate) = &self.inner.delegate else {
            return;
        };

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
        delegate.handle_event(event)
    }

    async fn vcard_did_change(&self, from: Jid, vcard: VCard4) -> Result<()> {
        debug!("New vcard for {} {:?}", from, vcard);

        let Some(profile): Option<UserProfile> = vcard.clone().try_into().ok() else {
            return Ok(());
        };

        let from = BareJid::from(from);

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

        let from = BareJid::from(from);

        self.inner
            .data_cache
            .insert_avatar_metadata(&from, &metadata)
            .await?;

        Ok(())

        // TODO: Fix this
        // match ctx
        //     .load_and_cache_avatar_image(&from, &metadata, CachePolicy::ReloadIgnoringCacheData)
        //     .await
        // {
        //     Ok(path) => {
        //         debug!("Finished downloading and caching image to {:?}", path);
        //         ctx.send_event(ClientEvent::ContactChanged { jid: from });
        //     }
        //     Err(err) => error!("Failed downloading and caching image. {}", err),
        // }
    }

    async fn presence_did_change(&self, presence: Presence) -> Result<()> {
        if presence.type_ == presence::Type::None {
            return Ok(());
        }

        let Some(from) = presence.from else {
            return Ok(());
        };

        let jid = BareJid::from(from);

        self.inner
            .data_cache
            .insert_presence(
                &jid,
                &types::Presence {
                    kind: Some(presence.type_.clone().into()),
                    show: presence.show.as_ref().map(|s| s.clone().into()),
                    status: presence.statuses.first_key_value().map(|kv| kv.1.clone()),
                },
            )
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
                    from: BareJid::from(from.clone()),
                });
            }
        }

        let message_is_carbon = message.is_carbon();
        let now = Utc::now();

        let parsed_message = match message {
            ReceivedMessage::Message(message) => MessageLike::try_from(TimestampedMessage {
                message,
                timestamp: now.into(),
            })?,
            ReceivedMessage::Carbon(carbon) => MessageLike::try_from(TimestampedMessage {
                message: carbon,
                timestamp: now.into(),
            })?,
        };

        debug!("Caching received message…");
        self.inner
            .data_cache
            .insert_messages([&parsed_message])
            .await?;

        let conversation = if message_is_carbon {
            &parsed_message.to
        } else {
            &parsed_message.from
        };
        self.send_event_for_message(conversation, &parsed_message);

        if let Some(chat_state) = chat_state {
            self.inner
                .data_cache
                .insert_chat_state(&chat_state.from, &chat_state.state)
                .await?;
            self.send_event(ClientEvent::ComposingUsersChanged {
                conversation: chat_state.from,
            })
        }

        // Don't send delivery receipts for carbons or anything other than a regular message.
        if message_is_carbon || !parsed_message.payload.is_message() {
            return Ok(());
        }

        let chat = self.client.get_mod::<mods::Chat>();
        chat.mark_message_received(parsed_message.id.clone(), parsed_message.from)?;

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
