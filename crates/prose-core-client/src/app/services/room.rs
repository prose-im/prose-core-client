// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use anyhow::{format_err, Result};
use chrono::Duration;
use tracing::{debug, info};

use crate::app::deps::{
    DynAppContext, DynClientEventDispatcher, DynDraftsRepository, DynMessageArchiveService,
    DynMessagesRepository, DynMessagingService, DynRoomAttributesService,
    DynRoomParticipationService, DynSidebarDomainService, DynTimeProvider,
    DynUserProfileRepository,
};
use crate::domain::messaging::models::{Emoji, Message, MessageId, MessageLike};
use crate::domain::rooms::models::{Room as DomainRoom, RoomAffiliation, RoomSpec};
use crate::domain::shared::models::{ParticipantId, ParticipantInfo, RoomId};
use crate::dtos::{
    Message as MessageDTO, MessageResultSet, MessageSender, RoomState, SendMessageRequest,
    StanzaId, UserBasicInfo, UserId,
};
use crate::{ClientEvent, ClientRoomEventType};

pub struct Room<Kind> {
    inner: Arc<RoomInner>,
    _type: PhantomData<Kind>,
}

pub struct DirectMessage;
pub struct Group;
pub struct Generic;

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

pub struct RoomInner {
    pub(crate) data: DomainRoom,

    pub(crate) ctx: DynAppContext,
    pub(crate) attributes_service: DynRoomAttributesService,
    pub(crate) client_event_dispatcher: DynClientEventDispatcher,
    pub(crate) drafts_repo: DynDraftsRepository,
    pub(crate) message_archive_service: DynMessageArchiveService,
    pub(crate) message_repo: DynMessagesRepository,
    pub(crate) messaging_service: DynMessagingService,
    pub(crate) participation_service: DynRoomParticipationService,
    pub(crate) sidebar_domain_service: DynSidebarDomainService,
    pub(crate) time_provider: DynTimeProvider,
    pub(crate) user_profile_repo: DynUserProfileRepository,
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
            .field("occupants", &self.data.participants())
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
            .participants()
            .iter()
            .map(ParticipantInfo::from)
            .collect()
    }
}

impl<Kind> Room<Kind> {
    pub async fn send_message(&self, request: SendMessageRequest) -> Result<()> {
        self.messaging_service
            .send_message(&self.data.room_id, &self.data.r#type, request)
            .await
    }

    pub async fn update_message(&self, id: MessageId, request: SendMessageRequest) -> Result<()> {
        self.messaging_service
            .update_message(&self.data.room_id, &self.data.r#type, &id, request)
            .await
    }

    pub async fn toggle_reaction_to_message(&self, id: MessageId, emoji: Emoji) -> Result<()> {
        let user_jid = ParticipantId::from(self.ctx.connected_id()?.to_user_id());
        let messages = self.message_repo.get(&self.data.room_id, &id).await?;

        let mut message = Message::reducing_messages(messages)
            .pop()
            .ok_or(format_err!("No message with id {}", id))?;

        message.toggle_reaction(&user_jid, emoji);
        let all_emojis = message
            .reactions_from(&user_jid)
            .cloned()
            .collect::<Vec<_>>();

        self.messaging_service
            .react_to_message(
                &self.data.room_id,
                &self.data.r#type,
                &id,
                all_emojis.as_slice(),
            )
            .await
    }

    pub async fn retract_message(&self, id: MessageId) -> Result<()> {
        self.messaging_service
            .retract_message(&self.data.room_id, &self.data.r#type, &id)
            .await
    }

    pub async fn load_messages_with_ids(&self, ids: &[MessageId]) -> Result<Vec<MessageDTO>> {
        let messages = self.message_repo.get_all(&self.data.room_id, ids).await?;
        Ok(self.reduce_messages_and_add_sender(messages).await)
    }

    pub async fn set_user_is_composing(&self, is_composing: bool) -> Result<()> {
        self.messaging_service
            .set_user_is_composing(&self.data.room_id, &self.data.r#type, is_composing)
            .await
    }

    pub async fn load_composing_users(&self) -> Result<Vec<UserBasicInfo>> {
        // If the chat state is 'composing' but older than 30 seconds we do not consider
        // the user as currently typing.
        let thirty_secs_ago = self.time_provider.now() - Duration::seconds(30);
        Ok(self.data.participants().composing_users(thirty_secs_ago))
    }

    pub async fn save_draft(&self, text: Option<&str>) -> Result<()> {
        self.drafts_repo.set(&self.data.room_id, text).await?;
        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);
        Ok(())
    }

    pub async fn load_draft(&self) -> Result<Option<String>> {
        self.drafts_repo.get(&self.data.room_id).await
    }

    pub async fn load_latest_messages(&self) -> Result<MessageResultSet> {
        debug!("Loading latest messages from server…");
        self.load_messages(None).await
    }

    pub async fn load_messages_before(&self, stanza_id: &StanzaId) -> Result<MessageResultSet> {
        debug!("Loading latest messages before '{stanza_id}' from server…");
        self.load_messages(Some(stanza_id)).await
    }
}

impl<Kind> Room<Kind> {
    async fn load_messages(&self, before: Option<&StanzaId>) -> Result<MessageResultSet> {
        let message_page_size = self.ctx.config.message_page_size;
        let max_message_pages_to_load = self.ctx.config.max_message_pages_to_load as usize;

        let mut messages = vec![];
        let mut last_message_id: Option<StanzaId> = before.cloned();
        let mut num_text_messages = 0;
        let mut loaded_pages = 0;

        while num_text_messages < message_page_size && loaded_pages < max_message_pages_to_load {
            let page = self
                .message_archive_service
                .load_messages(
                    &self.data.room_id,
                    &self.data.r#type,
                    last_message_id.as_ref(),
                    None,
                    message_page_size,
                )
                .await?;

            last_message_id = page.messages.first().map(|m| StanzaId::from(m.id.as_ref()));

            for archive_message in page.messages.into_iter().rev() {
                let Ok(parsed_message) = MessageLike::try_from(&archive_message) else {
                    continue;
                };

                if parsed_message.payload.is_message() {
                    num_text_messages += 1;
                }

                messages.push(parsed_message)
            }

            loaded_pages += 1;

            if page.is_last {
                last_message_id = None;
                break;
            }
        }

        debug!("Found {} messages. Saving to cache…", messages.len());
        self.message_repo
            .append(&self.data.room_id, messages.as_slice())
            .await?;

        let result_set = MessageResultSet {
            messages: self
                .reduce_messages_and_add_sender(messages.into_iter().rev())
                .await,
            last_message_id,
        };

        Ok(result_set)
    }

    async fn reduce_messages_and_add_sender(
        &self,
        messages: impl IntoIterator<Item = MessageLike>,
    ) -> Vec<MessageDTO> {
        let messages = Message::reducing_messages(messages);
        let mut message_dtos = Vec::with_capacity(messages.len());

        for message in messages {
            let from = self.resolve_message_sender(&message.from).await;

            message_dtos.push(MessageDTO {
                id: message.id,
                stanza_id: message.stanza_id,
                from,
                body: message.body,
                timestamp: message.timestamp,
                is_read: message.is_read,
                is_edited: message.is_edited,
                is_delivered: message.is_delivered,
                reactions: message.reactions,
                attachments: message.attachments,
            });
        }

        message_dtos
    }

    async fn resolve_message_sender(&self, id: &ParticipantId) -> MessageSender {
        let (name, mut real_id) = self
            .data
            .participants()
            .get(&id.clone().into())
            .map(|p| (p.name.clone(), p.real_id.clone()))
            .unwrap_or_else(|| (None, None));

        real_id = real_id.or_else(|| id.to_user_id());

        let sender_id = real_id
            .clone()
            .map(ParticipantId::from)
            .unwrap_or_else(|| id.clone());

        if let Some(name) = name {
            return MessageSender {
                id: sender_id,
                name,
            };
        }

        if let Some(real_id) = real_id {
            if let Some(name) = self
                .user_profile_repo
                .get_display_name(&real_id)
                .await
                .unwrap_or_default()
            {
                return MessageSender {
                    id: sender_id,
                    name,
                };
            }
        }

        let name = match id {
            ParticipantId::User(id) => id.formatted_username(),
            ParticipantId::Occupant(id) => id.formatted_nickname(),
        };
        MessageSender {
            id: sender_id,
            name,
        }
    }
}

impl Room<Group> {
    pub async fn resend_invites_to_members(&self) -> Result<()> {
        info!("Sending invites to group members…");

        let member_jids = self
            .data
            .participants()
            .values()
            .filter_map(|p| {
                if p.affiliation >= RoomAffiliation::Member {
                    p.real_id.clone()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        self.participation_service
            .invite_users_to_room(&self.data.room_id, member_jids.as_slice())
            .await?;
        Ok(())
    }

    pub async fn convert_to_private_channel(&self, name: impl AsRef<str>) -> Result<()> {
        self.sidebar_domain_service
            .reconfigure_item_with_spec(&self.data.room_id, RoomSpec::PrivateChannel, name.as_ref())
            .await?;
        Ok(())
    }
}

impl Room<PrivateChannel> {
    pub async fn convert_to_public_channel(&self) -> Result<()> {
        self.sidebar_domain_service
            .reconfigure_item_with_spec(
                &self.data.room_id,
                RoomSpec::PublicChannel,
                self.data.name().as_deref().unwrap_or_default(),
            )
            .await?;
        Ok(())
    }

    pub async fn invite_users(&self, users: impl IntoIterator<Item = &UserId>) -> Result<()> {
        let user_jids = users.into_iter().cloned().collect::<Vec<_>>();
        self.participation_service
            .invite_users_to_room(&self.data.room_id, user_jids.as_slice())
            .await?;
        Ok(())
    }
}

impl Room<PublicChannel> {
    pub async fn convert_to_private_channel(&self) -> Result<()> {
        self.sidebar_domain_service
            .reconfigure_item_with_spec(
                &self.data.room_id,
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
                .grant_membership(&self.data.room_id, user)
                .await?;
        }

        self.participation_service
            .invite_users_to_room(&self.data.room_id, user_jids.as_slice())
            .await?;
        Ok(())
    }
}

impl<Kind> Room<Kind>
where
    Kind: HasTopic,
{
    pub async fn set_topic(&self, topic: Option<String>) -> Result<()> {
        self.attributes_service
            .set_topic(&self.data.room_id, topic.as_deref())
            .await?;
        self.data.set_topic(topic);

        self.client_event_dispatcher
            .dispatch_room_event(self.data.clone(), ClientRoomEventType::AttributesChanged);

        Ok(())
    }
}

impl<Kind> Room<Kind>
where
    Kind: HasMutableName,
{
    pub async fn set_name(&self, name: impl AsRef<str>) -> Result<()> {
        self.sidebar_domain_service
            .rename_item(&self.data.room_id, name.as_ref())
            .await?;
        Ok(())
    }
}
