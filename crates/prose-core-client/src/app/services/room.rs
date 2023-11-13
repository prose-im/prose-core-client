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
use jid::{BareJid, Jid};
use tracing::{debug, info};

use crate::app::deps::{
    DynBookmarksService, DynClientEventDispatcher, DynDraftsRepository, DynMessageArchiveService,
    DynMessagesRepository, DynMessagingService, DynRoomAttributesService,
    DynRoomParticipationService, DynSidebarRepository, DynTimeProvider, DynUserProfileRepository,
};
use crate::domain::messaging::models::{Emoji, Message, MessageId, MessageLike};
use crate::domain::rooms::models::RoomInternals;
use crate::domain::shared::models::{RoomJid, RoomType};
use crate::domain::sidebar::models::{Bookmark, BookmarkType};
use crate::dtos::{
    Availability, Message as MessageDTO, MessageSender, UserBasicInfo, UserPresenceInfo,
};
use crate::util::jid_ext::{BareJidExt, JidExt};
use crate::ClientEvent;

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
    pub(crate) data: Arc<RoomInternals>,

    pub(crate) bookmarks_service: DynBookmarksService,
    pub(crate) client_event_dispatcher: DynClientEventDispatcher,
    pub(crate) drafts_repo: DynDraftsRepository,
    pub(crate) message_archive_service: DynMessageArchiveService,
    pub(crate) message_repo: DynMessagesRepository,
    pub(crate) messaging_service: DynMessagingService,
    pub(crate) participation_service: DynRoomParticipationService,
    pub(crate) sidebar_repo: DynSidebarRepository,
    pub(crate) time_provider: DynTimeProvider,
    pub(crate) attributes_service: DynRoomAttributesService,
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
            .field("jid", &self.data.info.jid)
            .field("name", &self.data.state.read().name)
            .field("description", &self.data.info.description)
            .field("user_jid", &self.data.info.user_jid)
            .field("user_nickname", &self.data.info.user_nickname)
            .field("subject", &self.data.state.read().subject)
            .field("occupants", &self.data.state.read().occupants)
            .finish_non_exhaustive()
    }
}

impl<Kind> PartialEq for Room<Kind> {
    fn eq(&self, other: &Self) -> bool {
        self.data.info.jid == other.data.info.jid
    }
}

impl<Kind> Room<Kind> {
    pub fn to_generic(&self) -> Room<Generic> {
        Room::from(self.inner.clone())
    }
}

impl<Kind> Room<Kind> {
    pub fn jid(&self) -> &RoomJid {
        &self.data.info.jid
    }

    pub fn name(&self) -> Option<String> {
        self.data.state.read().name.clone()
    }

    pub fn description(&self) -> Option<&str> {
        self.data.info.description.as_deref()
    }

    pub fn user_nickname(&self) -> &str {
        &self.data.info.user_nickname
    }

    pub fn subject(&self) -> Option<String> {
        self.data.state.read().subject.clone()
    }

    pub fn members(&self) -> Vec<UserPresenceInfo> {
        self.data
            .info
            .members
            .iter()
            .map(|(jid, member)| UserPresenceInfo {
                jid: jid.clone(),
                name: member.name.clone(),
                availability: Availability::Available,
            })
            .collect()
    }

    pub fn occupants(&self) -> Vec<UserBasicInfo> {
        self.data
            .state
            .read()
            .occupants
            .values()
            .filter_map(|occupant| {
                let Some(jid) = occupant.jid.clone() else {
                    return None;
                };
                let name = occupant
                    .name
                    .clone()
                    .unwrap_or_else(|| jid.to_display_name());
                Some(UserBasicInfo { jid, name })
            })
            .collect()
    }
}

impl<Kind> Room<Kind> {
    pub async fn send_message(&self, body: impl Into<String>) -> Result<()> {
        self.messaging_service
            .send_message(&self.data.info.jid, &self.data.info.room_type, body.into())
            .await
    }

    pub async fn update_message(&self, id: MessageId, body: impl Into<String>) -> Result<()> {
        self.messaging_service
            .update_message(
                &self.data.info.jid,
                &self.data.info.room_type,
                &id,
                body.into(),
            )
            .await
    }

    pub async fn toggle_reaction_to_message(&self, id: MessageId, emoji: Emoji) -> Result<()> {
        let messages = self.message_repo.get(&self.data.info.jid, &id).await?;

        let mut message = Message::reducing_messages(messages)
            .pop()
            .ok_or(format_err!("No message with id {}", id))?;

        message.toggle_reaction(&self.data.info.user_jid, emoji);
        let all_emojis = message
            .reactions_from(&self.data.info.user_jid)
            .cloned()
            .collect::<Vec<_>>();

        self.messaging_service
            .react_to_message(
                &self.data.info.jid,
                &self.data.info.room_type,
                &id,
                all_emojis.as_slice(),
            )
            .await
    }

    pub async fn retract_message(&self, id: MessageId) -> Result<()> {
        self.messaging_service
            .retract_message(&self.data.info.jid, &self.data.info.room_type, &id)
            .await
    }

    pub async fn load_messages_with_ids(&self, ids: &[&MessageId]) -> Result<Vec<MessageDTO>> {
        let messages = self.message_repo.get_all(&self.data.info.jid, ids).await?;
        Ok(self.reduce_messages_and_add_sender(messages).await)
    }

    pub async fn set_user_is_composing(&self, is_composing: bool) -> Result<()> {
        self.messaging_service
            .set_user_is_composing(&self.data.info.jid, &self.data.info.room_type, is_composing)
            .await
    }

    pub async fn load_composing_users(&self) -> Result<Vec<UserBasicInfo>> {
        // If the chat state is 'composing' but older than 30 seconds we do not consider
        // the user as currently typing.
        let thirty_secs_ago = self.time_provider.now() - Duration::seconds(30);
        Ok(self.data.state.read().composing_users(thirty_secs_ago))
    }

    pub async fn save_draft(&self, text: Option<&str>) -> Result<()> {
        self.drafts_repo.set(&self.data.info.jid, text).await
    }

    pub async fn load_draft(&self) -> Result<Option<String>> {
        self.drafts_repo.get(&self.data.info.jid).await
    }

    pub async fn load_latest_messages(&self) -> Result<Vec<MessageDTO>> {
        debug!("Loading messages from server…");

        let result = self
            .message_archive_service
            .load_messages(&self.data.info.jid, &self.data.info.room_type, None, None)
            .await?;

        let messages = result
            .0
            .iter()
            .map(|msg| MessageLike::try_from(msg))
            .collect::<Result<Vec<_>, _>>()?;

        debug!("Found {} messages. Saving to cache…", messages.len());
        self.message_repo
            .append(
                &self.data.info.jid,
                messages.iter().collect::<Vec<_>>().as_slice(),
            )
            .await?;

        Ok(self.reduce_messages_and_add_sender(messages).await)
    }
}

impl<Kind> Room<Kind> {
    async fn reduce_messages_and_add_sender(&self, messages: Vec<MessageLike>) -> Vec<MessageDTO> {
        let messages = Message::reducing_messages(messages);
        let mut message_dtos = Vec::with_capacity(messages.len());

        for message in messages {
            let name = self.resolve_user_name(&message.from).await;

            let from = MessageSender {
                jid: message.from.into_bare(),
                name,
            };

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
            });
        }

        message_dtos
    }

    async fn resolve_user_name(&self, jid: &Jid) -> String {
        let name = {
            let state = &*self.data.state.read();

            match jid {
                Jid::Bare(bare) => self
                    .data
                    .info
                    .members
                    .get(bare)
                    .map(|member| member.name.clone())
                    .or_else(|| state.occupants.get(jid).and_then(|o| o.name.clone())),
                Jid::Full(_) => state.occupants.get(jid).and_then(|o| o.name.clone()),
            }
        };

        if let Some(name) = name {
            return name;
        };

        if let Jid::Bare(bare) = &jid {
            if let Some(name) = self
                .user_profile_repo
                .get_display_name(bare)
                .await
                .unwrap_or_default()
            {
                return name;
            };
        }

        if self.data.info.room_type == RoomType::DirectMessage {
            jid.node_to_display_name()
        } else {
            jid.resource_to_display_name()
        }
    }
}

#[cfg(feature = "debug")]
impl<Kind> Room<Kind> {
    pub fn occupants_dbg(&self) -> Vec<crate::domain::rooms::models::Occupant> {
        self.data.state.read().occupants.values().cloned().collect()
    }
}

impl Room<Group> {
    pub async fn resend_invites_to_members(&self) -> Result<()> {
        info!("Sending invites to group members…");

        let member_jids = self
            .data
            .info
            .members
            .keys()
            .map(|jid| jid)
            .collect::<Vec<_>>();
        self.participation_service
            .invite_users_to_room(&self.data.info.jid, member_jids.as_slice())
            .await?;
        Ok(())
    }
}

impl<Kind> Room<Kind>
where
    Kind: Channel,
{
    pub async fn invite_users(&self, users: impl IntoIterator<Item = &BareJid>) -> Result<()> {
        let user_jids = users.into_iter().collect::<Vec<_>>();
        self.participation_service
            .invite_users_to_room(&self.data.info.jid, user_jids.as_slice())
            .await?;
        Ok(())
    }
}

impl<Kind> Room<Kind>
where
    Kind: HasTopic,
{
    pub async fn set_topic(&self, topic: Option<&str>) -> Result<()> {
        self.attributes_service
            .set_topic(&self.data.info.jid, topic)
            .await
    }
}

impl<Kind> Room<Kind>
where
    Kind: HasMutableName,
{
    pub async fn set_name(&self, name: impl AsRef<str>) -> Result<()> {
        if self
            .data
            .state
            .read()
            .name
            .as_ref()
            .map(|name| name.to_lowercase())
            == Some(name.as_ref().to_lowercase())
        {
            return Ok(());
        }

        self.attributes_service
            .set_name(&self.data.info.jid, name.as_ref())
            .await?;

        self.data.state.write().name = Some(name.as_ref().to_string());

        let bookmark_type = match self.data.info.room_type {
            RoomType::Pending => return Ok(()),
            RoomType::DirectMessage => BookmarkType::DirectMessage,
            RoomType::Group => BookmarkType::Group,
            RoomType::PrivateChannel => BookmarkType::PrivateChannel,
            RoomType::PublicChannel => BookmarkType::PublicChannel,
            RoomType::Generic => return Ok(()),
        };

        let mut bookmark = Bookmark {
            name: name.as_ref().to_string(),
            jid: self.data.info.jid.clone(),
            r#type: bookmark_type,
            is_favorite: false,
            in_sidebar: false,
        };

        let mut sidebar_changed = false;

        if let Some(mut sidebar_item) = self.sidebar_repo.get(&self.data.info.jid) {
            bookmark.in_sidebar = true;

            sidebar_item.name = name.as_ref().to_string();
            self.sidebar_repo.put(&sidebar_item);
            sidebar_changed = true;
        }

        self.bookmarks_service.save_bookmark(&bookmark).await?;

        if sidebar_changed {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::SidebarChanged);
        }

        Ok(())
    }
}
