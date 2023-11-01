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
    DynDraftsRepository, DynMessageArchiveService, DynMessagesRepository, DynMessagingService,
    DynRoomParticipationService, DynRoomTopicService, DynTimeProvider,
};
use crate::domain::messaging::models::{Emoji, Message, MessageId, MessageLike};
use crate::domain::rooms::models::RoomInternals;

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

impl HasTopic for Group {}
impl HasTopic for PrivateChannel {}
impl HasTopic for PublicChannel {}
impl HasTopic for Generic {}

pub struct RoomInner {
    pub(crate) data: Arc<RoomInternals>,

    pub(crate) time_provider: DynTimeProvider,
    pub(crate) messaging_service: DynMessagingService,
    pub(crate) message_archive_service: DynMessageArchiveService,
    pub(crate) participation_service: DynRoomParticipationService,
    pub(crate) topic_service: DynRoomTopicService,

    pub(crate) message_repo: DynMessagesRepository,
    pub(crate) drafts_repo: DynDraftsRepository,
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
            .field("name", &self.data.info.name)
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
    pub fn jid(&self) -> &BareJid {
        &self.data.info.jid
    }

    pub fn name(&self) -> Option<&str> {
        self.data.info.name.as_deref()
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

    pub fn members(&self) -> &[BareJid] {
        self.data.info.members.as_slice()
    }

    pub fn occupants(&self) -> Vec<BareJid> {
        self.data
            .state
            .read()
            .occupants
            .values()
            .filter_map(|occupant| occupant.jid.clone())
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

    pub async fn load_messages_with_ids(&self, ids: &[&MessageId]) -> Result<Vec<Message>> {
        let messages = self.message_repo.get_all(&self.data.info.jid, ids).await?;
        Ok(self.reduce_messages_and_lookup_real_jids(messages))
    }

    pub async fn set_user_is_composing(&self, is_composing: bool) -> Result<()> {
        self.messaging_service
            .set_user_is_composing(&self.data.info.jid, &self.data.info.room_type, is_composing)
            .await
    }

    pub async fn load_composing_users(&self) -> Result<Vec<BareJid>> {
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

    pub async fn load_latest_messages(&self) -> Result<Vec<Message>> {
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

        Ok(self.reduce_messages_and_lookup_real_jids(messages))
    }
}

impl<Kind> Room<Kind> {
    fn reduce_messages_and_lookup_real_jids(&self, mut messages: Vec<MessageLike>) -> Vec<Message> {
        let state = &*self.data.state.read();
        for message in messages.iter_mut() {
            if let Some(real_jid) = state
                .occupants
                .get(&message.from)
                .and_then(|o| o.jid.clone())
                .map(Jid::Bare)
            {
                message.from = real_jid;
            }
        }
        Message::reducing_messages(messages)
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

        let member_jids = self.data.info.members.iter().collect::<Vec<_>>();
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
        self.topic_service
            .set_topic(&self.data.info.jid, topic)
            .await
    }
}
