// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use anyhow::{anyhow, bail, ensure, format_err, Result};
use chrono::Duration;
use itertools::Itertools;
use tracing::{debug, error, info, warn};

use prose_markup::MarkdownParser;
use prose_xmpp::TimeProvider;

use crate::app::deps::{
    DynAppContext, DynClientEventDispatcher, DynDraftsRepository, DynEncryptionDomainService,
    DynMessageArchiveService, DynMessageIdProvider, DynMessagesRepository, DynMessagingService,
    DynRoomAttributesService, DynRoomParticipationService, DynSidebarDomainService,
    DynSyncedRoomSettingsService, DynTimeProvider, DynUserInfoDomainService,
};
use crate::domain::messaging::models::{
    send_message_request, ArchivedMessageRef, Emoji, Message, MessageId, MessageLike,
    MessageLikeBody, MessageLikeError, MessageParser, MessageRemoteId, MessageTargetId, ThreadId,
};
use crate::domain::messaging::models::{MessageLikePayload, SendMessageRequest};
use crate::domain::rooms::models::{Room as DomainRoom, RoomAffiliation, RoomSpec};
use crate::domain::settings::models::SyncedRoomSettings;
use crate::domain::shared::models::{
    AccountId, CachePolicy, MucId, ParticipantId, ParticipantInfo, RoomId, RoomType, StyledMessage,
};
use crate::domain::shared::utils::ContactNameBuilder;
use crate::dtos::{
    Mention, Message as MessageDTO, MessageFlags as MessageFlagsDTO, MessageResultSet,
    MessageSender, MessageServerId, ParticipantBasicInfo, Reaction as ReactionDTO,
    ReplyTo as ReplyToDTO, RoomState, SendMessageRequest as SendMessageRequestDTO, UserId, HTML,
};
use crate::infra::xmpp::util::MessageExt;
use crate::{ClientEvent, ClientRoomEventType};

pub struct Room<Kind> {
    inner: Arc<RoomInner>,
    _type: PhantomData<Kind>,
}

pub struct DirectMessage;
pub struct Group;
pub struct Generic;

#[allow(dead_code)]
pub trait Channel {}

pub struct PrivateChannel;
pub struct PublicChannel;

impl Channel for PrivateChannel {}
impl Channel for PublicChannel {}

pub trait HasTopic {}
pub trait HasMutableName {}

impl HasTopic for Group {}
impl HasTopic for PrivateChannel {}
impl HasTopic for PublicChannel {}
impl HasTopic for Generic {}

impl HasMutableName for PrivateChannel {}
impl HasMutableName for PublicChannel {}
impl HasMutableName for Generic {}

pub trait MucRoom {}

impl MucRoom for Group {}
impl MucRoom for PrivateChannel {}
impl MucRoom for PublicChannel {}
impl MucRoom for Generic {}

pub struct RoomInner {
    pub(crate) data: DomainRoom,

    pub(crate) attributes_service: DynRoomAttributesService,
    pub(crate) client_event_dispatcher: DynClientEventDispatcher,
    pub(crate) ctx: DynAppContext,
    pub(crate) drafts_repo: DynDraftsRepository,
    pub(crate) encryption_domain_service: DynEncryptionDomainService,
    pub(crate) message_archive_service: DynMessageArchiveService,
    pub(crate) message_id_provider: DynMessageIdProvider,
    pub(crate) message_repo: DynMessagesRepository,
    pub(crate) messaging_service: DynMessagingService,
    pub(crate) participation_service: DynRoomParticipationService,
    pub(crate) sidebar_domain_service: DynSidebarDomainService,
    pub(crate) synced_room_settings_service: DynSyncedRoomSettingsService,
    pub(crate) time_provider: DynTimeProvider,
    pub(crate) user_info_domain_service: DynUserInfoDomainService,
}

impl<Kind> From<Arc<RoomInner>> for Room<Kind> {
    fn from(inner: Arc<RoomInner>) -> Self {
        Room {
            inner,
            _type: Default::default(),
        }
    }
}

impl<Kind> Clone for Room<Kind> {
    fn clone(&self) -> Self {
        Self::from(self.inner.clone())
    }
}

impl<Kind> Deref for Room<Kind> {
    type Target = RoomInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Kind> Debug for Room<Kind> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Room")
            .field("jid", &self.data.room_id)
            .field("name", &self.data.name())
            .field("description", &self.data.description())
            .field("user_nickname", &self.data.user_nickname)
            .field("subject", &self.data.topic())
            .field("occupants", &self.data.with_participants(|p| p.clone()))
            .finish_non_exhaustive()
    }
}

impl<Kind> PartialEq for Room<Kind> {
    fn eq(&self, other: &Self) -> bool {
        self.data.room_id == other.data.room_id
    }
}

impl<Kind> Room<Kind> {
    pub fn to_generic(&self) -> Room<Generic> {
        Room::from(self.inner.clone())
    }
}

impl<Kind> Room<Kind> {
    pub fn jid(&self) -> &RoomId {
        &self.data.room_id
    }

    pub fn state(&self) -> RoomState {
        self.data.state()
    }

    pub fn name(&self) -> Option<String> {
        self.data.name()
    }

    pub fn description(&self) -> Option<String> {
        self.data.description()
    }

    pub fn user_nickname(&self) -> &str {
        &self.data.user_nickname
    }

    pub fn subject(&self) -> Option<String> {
        self.data.topic()
    }

    pub fn participants(&self) -> Vec<ParticipantInfo> {
        self.data
            .with_participants(|p| p.iter().map(ParticipantInfo::from).collect())
    }
}

impl<Kind> Room<Kind> {
    pub async fn send_message(&self, request: SendMessageRequestDTO) -> Result<()> {
        ensure!(!request.is_empty(), "SendMessageRequest is empty");

        // Handle (temporary) slash commands…
        match request.body.as_ref().map(|body| body.text.as_ref()) {
            Some("/omemo enable") => {
                self.set_encryption_enabled(true).await;
                self.show_system_message("OMEMO is now enabled.").await?;
                return Ok(());
            }
            Some("/omemo disable") => {
                self.set_encryption_enabled(false).await;
                self.show_system_message("OMEMO is now disabled.").await?;
                return Ok(());
            }
            _ => (),
        }

        self.process_send_message_request(
            &self.ctx.connected_account()?,
            request,
            ProcessMessageAction::Send,
        )
        .await
    }

    pub async fn reply_to_message(
        &self,
        id: MessageId,
        request: SendMessageRequestDTO,
    ) -> Result<()> {
        ensure!(!request.is_empty(), "SendMessageRequest is empty");

        let account = self.ctx.connected_account()?;

        let Some(message) = self
            .message_repo
            .get(&account, &self.data.room_id, &id)
            .await?
            .into_iter()
            .next()
        else {
            bail!("Could not find message to reply to.")
        };

        let MessageLikePayload::Message { thread_id, .. } = message.payload else {
            bail!("Message is not a regular message and cannot be replied to.")
        };

        let thread_id = thread_id.unwrap_or_else(|| ThreadId::from(message.id.into_inner()));

        self.process_send_message_request(
            &self.ctx.connected_account()?,
            request,
            ProcessMessageAction::ReplyInThread { thread_id },
        )
        .await
    }

    pub async fn update_message(
        &self,
        id: MessageId,
        request: SendMessageRequestDTO,
    ) -> Result<()> {
        ensure!(!request.is_empty(), "SendMessageRequest is empty");

        let account = self.ctx.connected_account()?;

        // Resolve the external ID of the message…
        let Some(target_id) = self
            .message_repo
            .resolve_message_id_to_remote_id(&account, &self.data.room_id, &id)
            .await?
        else {
            bail!("Failed to resolve message id '{id}' to a server id")
        };

        self.process_send_message_request(
            &account,
            request,
            ProcessMessageAction::Update {
                target_message_id: id,
                target_remote_id: target_id,
            },
        )
        .await
    }

    pub async fn toggle_reaction_to_message(&self, id: MessageId, emoji: Emoji) -> Result<()> {
        let account = self.ctx.connected_account()?;
        let messages = self
            .message_repo
            .get(&account, &self.data.room_id, &id)
            .await?;
        let user_jid = ParticipantId::from(account.into_user_id());

        let mut message = Message::reducing_messages(messages)
            .pop()
            .ok_or(format_err!("No message with id {}", id))?;

        message.toggle_reaction(&user_jid, emoji);
        let all_emojis = message
            .reactions_from(&user_jid)
            .cloned()
            .collect::<Vec<_>>();

        match &self.data.room_id {
            RoomId::User(room_id) => {
                let Some(remote_id) = &message.remote_id else {
                    bail!("Cannot react to message for which we do not have a RemoteId.")
                };
                self.messaging_service
                    .react_to_chat_message(room_id, remote_id, &all_emojis)
                    .await
            }
            RoomId::Muc(room_id) => {
                let Some(stanza_id) = &message.server_id else {
                    bail!("Cannot react to MUC message for which we do not have a StanzaId.")
                };
                self.messaging_service
                    .react_to_muc_message(room_id, stanza_id, &all_emojis)
                    .await
            }
        }
    }

    pub async fn retract_message(&self, id: MessageRemoteId) -> Result<()> {
        self.messaging_service
            .retract_message(&self.data.room_id, &id)
            .await
    }

    pub async fn load_messages_with_ids(&self, ids: &[MessageId]) -> Result<Vec<MessageDTO>> {
        let account = self.ctx.connected_account()?;
        let messages = self
            .message_repo
            .get_all(&account, &self.data.room_id, ids)
            .await?;
        Ok(self
            .reduce_messages_and_add_sender(&account, messages)
            .await)
    }

    pub async fn set_user_is_composing(&self, is_composing: bool) -> Result<()> {
        self.messaging_service
            .set_user_is_composing(&self.data.room_id, is_composing)
            .await
    }

    pub async fn load_composing_users(&self) -> Result<Vec<ParticipantBasicInfo>> {
        // If the chat state is 'composing' but older than 30 seconds we do not consider
        // the user as currently typing.
        let thirty_secs_ago = self.time_provider.now() - Duration::seconds(30);
        Ok(self
            .data
            .with_participants(|p| p.composing_users(thirty_secs_ago)))
    }

    pub async fn save_draft(&self, text: Option<&str>) -> Result<()> {
        self.drafts_repo
            .set(&self.ctx.connected_account()?, &self.data.room_id, text)
            .await?;
        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);
        Ok(())
    }

    pub async fn load_draft(&self) -> Result<Option<String>> {
        self.drafts_repo
            .get(&self.ctx.connected_account()?, &self.data.room_id)
            .await
    }

    pub async fn load_latest_messages(&self) -> Result<MessageResultSet> {
        debug!("Loading latest messages from server…");
        let messages = self.load_messages(None).await?;
        Ok(messages)
    }

    pub async fn load_messages_before(
        &self,
        stanza_id: &MessageServerId,
    ) -> Result<MessageResultSet> {
        debug!("Loading latest messages before '{stanza_id}' from server…");
        self.load_messages(Some(stanza_id)).await
    }

    pub async fn load_unread_messages(&self) -> Result<MessageResultSet> {
        let Some(last_read_message) = self.data.settings().last_read_message.clone() else {
            return self.load_latest_messages().await;
        };

        let account = self.ctx.connected_account()?;

        let messages = self
            .message_repo
            .get_messages_after(&account, &self.data.room_id, last_read_message.timestamp)
            .await?;

        Ok(MessageResultSet {
            messages: self
                .reduce_messages_and_add_sender(&account, messages)
                .await,
            last_message_id: None,
        })
    }

    pub async fn set_last_read_message(&self, id: &MessageId) -> Result<()> {
        let account = self.ctx.connected_account()?;

        let mut messages = self
            .message_repo
            .get(&account, &self.data.room_id, id)
            .await?;

        if messages.is_empty() {
            return Err(anyhow!("No message exists with id {id}."));
        }

        let message = messages.swap_remove(0);

        if let Some(stanza_id) = message.server_id {
            return self
                .set_last_read_message_ref(
                    &account,
                    Some(ArchivedMessageRef {
                        stanza_id,
                        timestamp: message.timestamp,
                    }),
                    true,
                )
                .await;
        }

        self.set_last_read_message_ref(
            &account,
            self.message_repo
                .get_last_received_message(&account, &self.data.room_id, Some(message.timestamp))
                .await?,
            true,
        )
        .await?;

        Ok(())
    }

    pub async fn mark_as_read(&self) -> Result<()> {
        let account = self.ctx.connected_account()?;

        let Some(message_ref) = self
            .message_repo
            .get_last_received_message(&account, &self.data.room_id, None)
            .await?
        else {
            return Ok(());
        };

        self.set_last_read_message_ref(&account, Some(message_ref), false)
            .await?;
        Ok(())
    }

    pub fn encryption_enabled(&self) -> bool {
        self.data.settings().encryption_enabled
    }

    pub async fn set_encryption_enabled(&self, enabled: bool) {
        self.update_synced_settings(|settings| settings.encryption_enabled = enabled)
            .await
    }
}

pub enum ProcessMessageAction {
    Send,
    ReplyInThread {
        thread_id: ThreadId,
    },
    Update {
        target_message_id: MessageId,
        target_remote_id: MessageRemoteId,
    },
}

impl<Kind> Room<Kind> {
    async fn process_send_message_request(
        &self,
        account: &AccountId,
        request: SendMessageRequestDTO,
        action: ProcessMessageAction,
    ) -> Result<()> {
        let mut message_body = MessageLikeBody::default();
        let mut message_request = SendMessageRequest {
            id: self.message_id_provider.new_id(),
            body: None,
            attachments: request.attachments.clone(),
        };

        // Process message body if there is one…
        if let Some(body) = request.body {
            // Parse markdown…
            let parser = MarkdownParser::new(body.text.as_ref());
            let html = HTML::new(parser.convert_to_html());
            let fallback = StyledMessage::new(parser.convert_to_message_styling());
            let mentions = parser
                .collect_mentions()
                .into_iter()
                .map(|jid| Mention {
                    user: jid.into(),
                    range: None,
                })
                .collect::<Vec<_>>();

            message_body = MessageLikeBody {
                raw: body.text.to_string(),
                html,
                mentions: mentions.clone(),
            };

            // Encrypt message if needed…
            let payload = match self.data.r#type {
                RoomType::DirectMessage | RoomType::Group | RoomType::PrivateChannel
                    if self.data.settings().encryption_enabled =>
                {
                    let user_ids = self.data.with_participants(|p| {
                        p.iter()
                            .filter_map(|(_, participant)| {
                                if participant.is_self {
                                    return None;
                                }
                                participant.real_id.clone()
                            })
                            .sorted()
                            .collect::<Vec<_>>()
                    });

                    send_message_request::Payload::Encrypted(
                        self.encryption_domain_service
                            .encrypt_message(user_ids, fallback.into_string())
                            .await?,
                    )
                }
                _ => send_message_request::Payload::Unencrypted {
                    message: body.text,
                    fallback,
                },
            };

            message_request.body = Some(send_message_request::Body { payload, mentions });
        }

        // Build appropriate payload…
        let payload = match &action {
            ProcessMessageAction::Send => MessageLikePayload::Message {
                body: message_body,
                attachments: request.attachments,
                encryption_info: None,
                is_transient: false,
                reply_to: None,
                thread_id: None,
            },
            ProcessMessageAction::ReplyInThread { thread_id } => MessageLikePayload::Message {
                body: message_body,
                attachments: request.attachments,
                encryption_info: None,
                is_transient: false,
                reply_to: None,
                thread_id: Some(thread_id.clone()),
            },
            ProcessMessageAction::Update {
                target_remote_id, ..
            } => MessageLikePayload::Correction {
                target_id: target_remote_id.clone().into(),
                body: message_body,
                attachments: request.attachments,
                encryption_info: None,
            },
        };

        // Save the unencrypted message so that we can look it up later…
        let message_id = message_request.id.clone();
        self.message_repo
            .append(
                &account,
                &self.data.room_id,
                &[MessageLike {
                    id: message_id.clone(),
                    remote_id: Some(message_id.to_string().into()),
                    server_id: None,
                    to: None,
                    from: account.to_user_id().into(),
                    timestamp: self.time_provider.now(),
                    payload,
                }],
            )
            .await?;

        // Pass message to MessagingService and build a ClientRoomEvent…
        let event = match action {
            ProcessMessageAction::Send => {
                self.messaging_service
                    .send_message(&self.data.room_id, message_request)
                    .await?;
                ClientRoomEventType::MessagesAppended {
                    message_ids: vec![message_id],
                }
            }
            ProcessMessageAction::ReplyInThread { thread_id } => {
                self.messaging_service
                    .send_message_to_thread(&self.data.room_id, &thread_id, message_request)
                    .await?;
                // TODO: Add parent message to this event?
                ClientRoomEventType::MessagesAppended {
                    message_ids: vec![message_id],
                }
            }
            ProcessMessageAction::Update {
                target_message_id,
                target_remote_id,
            } => {
                self.messaging_service
                    .update_message(&self.data.room_id, &target_remote_id, message_request)
                    .await?;
                ClientRoomEventType::MessagesUpdated {
                    message_ids: vec![target_message_id],
                }
            }
        };

        // Dispatch event to notify UI about changes…
        self.client_event_dispatcher
            .dispatch_room_event(self.data.clone(), event);

        Ok(())
    }

    async fn load_messages(&self, before: Option<&MessageServerId>) -> Result<MessageResultSet> {
        let account = self.ctx.connected_account()?;
        let message_page_size = self.ctx.config.message_page_size;
        let max_message_pages_to_load = self.ctx.config.max_message_pages_to_load as usize;

        let mut messages = vec![];
        let mut last_message_id: Option<MessageServerId> = before.cloned();
        let mut num_text_messages = 0;
        let mut text_message_ids = vec![];
        let mut loaded_pages = 0;

        while num_text_messages < message_page_size && loaded_pages < max_message_pages_to_load {
            let page = self
                .message_archive_service
                .load_messages_before(
                    &self.data.room_id,
                    last_message_id.as_ref(),
                    message_page_size,
                )
                .await?;

            last_message_id = page
                .messages
                .first()
                .map(|m| MessageServerId::from(m.id.as_ref()));

            // We're potentially loading multiple pages all oldest from newest, i.e.:
            // Page 1: 4, 5, 6
            // Page 2: 1, 2, 3
            // and we want to push them into `messages` in the order 6, 5, 4, 3, 2, 1 which is
            // why we need to iterate over each page in reverse…
            for archive_message in page.messages.into_iter().rev() {
                let inner_message = archive_message.forwarded.stanza.as_ref();

                let is_our_message = inner_message
                    .and_then(|m| m.sender())
                    .map(|s| self.data.is_current_user(&account, &s.to_participant_id()))
                    .unwrap_or_default();

                let message_id = if is_our_message {
                    if let Some(remote_id) = inner_message.and_then(|m| m.id.clone()) {
                        self.message_repo
                            .resolve_remote_id_to_message_id(
                                &account,
                                &self.data.room_id,
                                &MessageRemoteId::from(remote_id),
                            )
                            .await
                            .unwrap_or_default()
                    } else {
                        None
                    }
                } else {
                    self.message_repo
                        .resolve_server_id_to_message_id(
                            &account,
                            &self.data.room_id,
                            &MessageServerId::from(archive_message.id.as_ref()),
                        )
                        .await
                        .unwrap_or_default()
                }
                .unwrap_or_else(|| self.message_id_provider.new_id());

                let parsed_message = match MessageParser::new(
                    message_id,
                    Some(self.data.clone()),
                    Default::default(),
                    self.encryption_domain_service.clone(),
                    None,
                )
                .parse_mam_message(archive_message)
                .await
                {
                    Ok(message) => message,
                    Err(error) => {
                        match error.downcast_ref::<MessageLikeError>() {
                            Some(MessageLikeError::NoPayload) => (),
                            None => {
                                error!("Failed to parse MAM message. {}", error.to_string());
                            }
                        }
                        continue;
                    }
                };

                // Skip archived error messages. These usually don't have a message id, so the web
                // frontend chokes on that. And what's the point of archiving an error
                // message really?
                if parsed_message.payload.is_error() {
                    continue;
                }

                if parsed_message.payload.is_message() {
                    num_text_messages += 1;

                    if let Some(remote_id) = parsed_message.remote_id.clone() {
                        text_message_ids.push(MessageTargetId::RemoteId(remote_id))
                    }
                    if let Some(stanza_id) = parsed_message.server_id.clone() {
                        text_message_ids.push(MessageTargetId::ServerId(stanza_id))
                    }
                }

                messages.push(parsed_message)
            }

            loaded_pages += 1;

            if page.is_last {
                last_message_id = None;
                break;
            }
        }

        let later_targeting_earlier_messages = if before.is_some() && !text_message_ids.is_empty() {
            // We want to only load messages that are newer than our newest message, since we might
            // have older messages in our cache from previous runs and these could mess up the
            // order if we'll append them to the end of our array for reducing.
            // Note that `messages` is currently sorted from newest to oldest.
            if let Some(newest_message_timestamp) = messages.first().as_ref().map(|m| m.timestamp) {
                self.message_repo
                    .get_messages_targeting(
                        &account,
                        &self.data.room_id,
                        &text_message_ids,
                        &newest_message_timestamp,
                    )
                    .await?
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        debug!("Found {} messages. Saving to cache…", messages.len());
        self.message_repo
            .append(&account, &self.data.room_id, &messages)
            .await?;

        // So we have our `messages` in the order from newest to oldest (6, 5, 4, …) and need
        // them in the order from oldest to newest (1, 2, 3, …) to reduce and return them.
        // `later_targeting_earlier_messages` is already in the order from oldest to newest and
        // is guaranteed to only contain messages newer than those in `messages`. So we'll flip
        // `messages`, chain `later_targeting_earlier_messages` and everything should be fine
        // and dandy…
        let result_set = MessageResultSet {
            messages: self
                .reduce_messages_and_add_sender(
                    &account,
                    messages
                        .into_iter()
                        .rev()
                        .chain(later_targeting_earlier_messages.into_iter()),
                )
                .await,
            last_message_id,
        };

        Ok(result_set)
    }

    async fn reduce_messages_and_add_sender(
        &self,
        account: &AccountId,
        messages: impl IntoIterator<Item = MessageLike>,
    ) -> Vec<MessageDTO> {
        let messages = Message::reducing_messages(messages);
        let mut message_dtos = Vec::with_capacity(messages.len());
        let mut message_senders = HashMap::new();
        let last_read_message_id = self
            .data
            .settings()
            .last_read_message
            .as_ref()
            .map(|msg| msg.stanza_id.clone());

        async fn resolve_message_sender<'a, Kind>(
            room: &Room<Kind>,
            id: Cow<'a, ParticipantId>,
            map: &mut HashMap<ParticipantId, MessageSender>,
        ) -> MessageSender {
            if let Some(sender) = map.get(id.as_ref()) {
                return sender.clone();
            };
            let sender = room.resolve_message_sender(id.as_ref()).await;
            map.insert(id.into_owned(), sender.clone());
            sender
        }

        for message in messages {
            let from =
                resolve_message_sender(self, Cow::Borrowed(&message.from), &mut message_senders)
                    .await;

            let mut reactions = vec![];
            for reaction in message.reactions {
                let mut from = vec![];

                for sender in reaction.from {
                    from.push(
                        resolve_message_sender(self, Cow::Owned(sender), &mut message_senders)
                            .await,
                    );
                }

                reactions.push(ReactionDTO {
                    emoji: reaction.emoji,
                    from,
                })
            }

            let is_last_read_message =
                message.server_id.is_some() && message.server_id == last_read_message_id;

            let reply_to = 'outer: {
                if let Some(reply_to) = message.reply_to {
                    let message_id = match reply_to.id {
                        MessageTargetId::ServerId(server_id) => {
                            self.message_repo
                                .resolve_server_id_to_message_id(
                                    &account,
                                    &self.data.room_id,
                                    &server_id,
                                )
                                .await
                        }
                        MessageTargetId::RemoteId(remote_id) => {
                            self.message_repo
                                .resolve_remote_id_to_message_id(
                                    &account,
                                    &self.data.room_id,
                                    &remote_id,
                                )
                                .await
                        }
                    }
                    .unwrap_or_default();

                    let replied_to_message = if let Some(message_id) = &message_id {
                        let messages = self
                            .message_repo
                            .get(&account, &self.data.room_id, message_id)
                            .await
                            .unwrap_or_default();
                        Message::reducing_messages(messages).into_iter().next()
                    } else {
                        None
                    };

                    let (replied_to_sender, body, timestamp) = replied_to_message
                        .map(|m| (Some(m.from), Some(m.body.raw), Some(m.timestamp)))
                        .unwrap_or_else(|| (reply_to.to, reply_to.quote, None));

                    let Some(replied_to_sender) = replied_to_sender else {
                        break 'outer None;
                    };

                    Some(ReplyToDTO {
                        id: message_id,
                        sender: resolve_message_sender(
                            self,
                            Cow::Borrowed(&replied_to_sender),
                            &mut message_senders,
                        )
                        .await,
                        timestamp,
                        body,
                    })
                } else {
                    None
                }
            };

            message_dtos.push(MessageDTO {
                id: message.id,
                from,
                body: message.body,
                timestamp: message.timestamp,
                flags: MessageFlagsDTO {
                    is_read: message.flags.is_read,
                    is_edited: message.flags.is_edited,
                    is_delivered: message.flags.is_delivered,
                    is_transient: message.flags.is_transient,
                    is_encrypted: message.flags.is_encrypted,
                    is_last_read: is_last_read_message,
                },
                reactions,
                attachments: message.attachments,
                mentions: message.mentions,
                reply_to,
            });
        }

        message_dtos
    }

    async fn resolve_message_sender(&self, id: &ParticipantId) -> MessageSender {
        let (name, avatar, mut real_id) = self
            .data
            .with_participants(|p| {
                p.get(id).map(|p| {
                    (
                        Some(p.name().unwrap_or_participant_id(id)),
                        p.avatar.clone(),
                        p.real_id.clone(),
                    )
                })
            })
            .unwrap_or_else(|| (None, None, None));

        if let Some(name) = name {
            return MessageSender {
                id: id.clone(),
                name,
                avatar,
            };
        }

        real_id = real_id.or_else(|| id.to_user_id());

        let Some(real_id) = real_id else {
            return MessageSender {
                id: id.clone(),
                name: ContactNameBuilder::new().unwrap_or_participant_id(id),
                avatar,
            };
        };

        let (name, avatar) = self
            .user_info_domain_service
            .get_user_info(&real_id, CachePolicy::ReturnCacheDataDontLoad)
            .await
            .unwrap_or_default()
            .map(|i| (i.display_name().unwrap_or_participant_id(id), i.avatar))
            .unwrap_or_else(|| (ContactNameBuilder::new().unwrap_or_participant_id(id), None));

        MessageSender {
            id: id.clone(),
            name,
            avatar,
        }
    }

    async fn show_system_message(&self, message: impl Into<String>) -> Result<()> {
        let id = self.message_id_provider.new_id();
        let message = message.into();

        self.message_repo
            .append(
                &self.ctx.connected_account()?,
                &self.data.room_id,
                &[MessageLike {
                    id: id.clone(),
                    remote_id: Some(id.to_string().into()),
                    server_id: None,
                    to: None,
                    from: ParticipantId::User("prose-bot@prose.org".parse()?),
                    timestamp: self.time_provider.now(),
                    payload: MessageLikePayload::Message {
                        body: MessageLikeBody {
                            raw: message.clone().into(),
                            html: message.into(),
                            mentions: vec![],
                        },
                        attachments: vec![],
                        encryption_info: None,
                        is_transient: true,
                        reply_to: None,
                        thread_id: None,
                    },
                }],
            )
            .await?;

        self.client_event_dispatcher.dispatch_room_event(
            self.data.clone(),
            ClientRoomEventType::MessagesAppended {
                message_ids: vec![id.into()],
            },
        );

        Ok(())
    }

    async fn update_synced_settings(&self, handler: impl FnOnce(&mut SyncedRoomSettings)) {
        let updated_settings = {
            let (settings_changed, updated_settings) = self.data.with_settings_mut(|settings| {
                let mut updated_settings = settings.clone();
                handler(&mut updated_settings);
                let settings_changed = &updated_settings != settings;
                *settings = updated_settings.clone();
                (settings_changed, updated_settings)
            });

            if !settings_changed {
                return;
            }

            updated_settings
        };

        match self
            .synced_room_settings_service
            .save_settings(&self.data.room_id, &updated_settings)
            .await
        {
            Ok(_) => (),
            Err(err) => {
                error!("Failed to save updated room settings. {}", err.to_string())
            }
        }
    }

    async fn set_last_read_message_ref(
        &self,
        account: &AccountId,
        message_ref: Option<ArchivedMessageRef>,
        send_message_changed_events: bool,
    ) -> Result<()> {
        let mut updated_server_ids = vec![];

        self.update_synced_settings(|settings| {
            if settings.last_read_message == message_ref {
                return;
            }

            if let Some(former_message_ref) = settings.last_read_message.take() {
                updated_server_ids.push(former_message_ref.stanza_id);
            }

            if let Some(stanza_id) = message_ref.as_ref().map(|r| r.stanza_id.clone()) {
                updated_server_ids.push(stanza_id);
            }

            settings.last_read_message = message_ref;
        })
        .await;

        if updated_server_ids.is_empty() {
            return Ok(());
        }

        if send_message_changed_events {
            let mut updated_message_ids = vec![];

            for id in updated_server_ids {
                if let Some(message_id) = self
                    .message_repo
                    .resolve_server_id_to_message_id(&account, &self.data.room_id, &id)
                    .await?
                {
                    updated_message_ids.push(message_id);
                }
            }

            if !updated_message_ids.is_empty() {
                self.client_event_dispatcher.dispatch_room_event(
                    self.data.clone(),
                    ClientRoomEventType::MessagesUpdated {
                        message_ids: updated_message_ids,
                    },
                )
            }
        }

        self.inner.data.set_needs_update_statistics();
        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }
}

impl Room<Group> {
    pub async fn resend_invites_to_members(&self) -> Result<()> {
        info!("Sending invites to group members…");

        let member_jids = self.data.with_participants(|p| {
            p.values()
                .filter_map(|p| {
                    if p.affiliation >= RoomAffiliation::Member {
                        p.real_id.clone()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        });

        self.participation_service
            .invite_users_to_room(self.muc_id(), member_jids.as_slice())
            .await?;
        Ok(())
    }

    pub async fn convert_to_private_channel(&self, name: impl AsRef<str>) -> Result<()> {
        self.sidebar_domain_service
            .reconfigure_item_with_spec(self.muc_id(), RoomSpec::PrivateChannel, name.as_ref())
            .await?;
        Ok(())
    }
}

impl Room<PrivateChannel> {
    pub async fn convert_to_public_channel(&self) -> Result<()> {
        self.sidebar_domain_service
            .reconfigure_item_with_spec(
                self.muc_id(),
                RoomSpec::PublicChannel,
                self.data.name().as_deref().unwrap_or_default(),
            )
            .await?;
        Ok(())
    }

    pub async fn invite_users(&self, users: impl IntoIterator<Item = &UserId>) -> Result<()> {
        let user_jids = users.into_iter().cloned().collect::<Vec<_>>();
        self.participation_service
            .invite_users_to_room(self.muc_id(), user_jids.as_slice())
            .await?;
        Ok(())
    }
}

impl Room<PublicChannel> {
    pub async fn convert_to_private_channel(&self) -> Result<()> {
        self.sidebar_domain_service
            .reconfigure_item_with_spec(
                self.muc_id(),
                RoomSpec::PrivateChannel,
                self.data.name().as_deref().unwrap_or_default(),
            )
            .await?;
        Ok(())
    }

    pub async fn invite_users(&self, users: impl IntoIterator<Item = &UserId>) -> Result<()> {
        let user_jids = users.into_iter().cloned().collect::<Vec<_>>();

        for user in user_jids.iter() {
            self.participation_service
                .grant_membership(self.muc_id(), user)
                .await?;
        }

        self.participation_service
            .invite_users_to_room(self.muc_id(), user_jids.as_slice())
            .await?;
        Ok(())
    }
}

impl<Kind> Room<Kind>
where
    Kind: MucRoom,
{
    pub fn muc_id(&self) -> &MucId {
        self.data
            .room_id
            .muc_id()
            .expect("MucRoom must have RoomId::Muc")
    }
}

impl<Kind> Room<Kind>
where
    Kind: HasTopic,
{
    pub async fn set_topic(&self, topic: Option<String>) -> Result<()> {
        let room_id = self
            .data
            .room_id
            .muc_id()
            .ok_or_else(|| anyhow!("Cannot set topic on non-MUC room"))?;

        self.attributes_service
            .set_topic(room_id, topic.as_deref())
            .await?;
        self.data.set_topic(topic);

        self.client_event_dispatcher
            .dispatch_room_event(self.data.clone(), ClientRoomEventType::AttributesChanged);

        Ok(())
    }
}

impl<Kind> Room<Kind>
where
    Kind: HasMutableName + MucRoom,
{
    pub async fn set_name(&self, name: impl AsRef<str>) -> Result<()> {
        self.sidebar_domain_service
            .rename_item(&self.muc_id(), name.as_ref())
            .await?;
        Ok(())
    }
}
