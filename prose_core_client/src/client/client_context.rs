use std::path::PathBuf;
use std::sync::Arc;

use jid::{BareJid, Jid};
use tokio::sync::RwLock;
use tracing::info;

use prose_core_lib::modules::profile::avatar::ImageId;
use prose_core_lib::modules::{ArchivedMessage, Caps, Chat, Fin, Profile, Roster, MAM};
use prose_core_lib::stanza::message;
use prose_core_lib::stanza::message::ChatState;
use prose_core_lib::ConnectedClient;
use prose_macros::with_xmpp_client;

use crate::cache::{AvatarCache, DataCache, IMAGE_OUTPUT_MIME_TYPE, MAX_IMAGE_DIMENSIONS};
use crate::client::ClientError;
use crate::domain_ext::UserProfile;
use crate::types::message_like::Payload;
use crate::types::{AvatarMetadata, Capabilities, MessageLike, RosterItem};
use crate::{ClientDelegate, ClientEvent};

pub(crate) struct ClientContext<D: DataCache, A: AvatarCache> {
    pub capabilities: Capabilities,
    pub xmpp: RwLock<Option<XMPPClient>>,
    pub delegate: Option<Arc<Box<dyn ClientDelegate>>>,
    pub data_cache: D,
    pub avatar_cache: A,
}

pub(crate) struct XMPPClient {
    pub jid: BareJid,
    pub client: ConnectedClient,
    pub roster: Arc<Roster>,
    pub profile: Arc<Profile>,
    pub chat: Arc<Chat>,
    pub mam: Arc<MAM>,
    pub caps: Arc<Caps>,
}

impl<D: DataCache, A: AvatarCache> ClientContext<D, A> {
    pub fn send_event(&self, event: ClientEvent) {
        if let Some(delegate) = &self.delegate {
            delegate.handle_event(event)
        }
    }

    pub fn send_event_for_message(&self, conversation: &BareJid, message: &MessageLike) {
        let Some(delegate) = &self.delegate else {
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
}

impl<D: DataCache, A: AvatarCache> ClientContext<D, A> {
    pub async fn load_and_cache_avatar_image(
        &self,
        from: &BareJid,
        metadata: &AvatarMetadata,
    ) -> anyhow::Result<Option<PathBuf>> {
        if let Some(cached_image) = self
            .avatar_cache
            .cached_avatar_image_url(&from, &metadata.checksum)
        {
            info!("Found cached image for {}", from);
            return Ok(Some(cached_image));
        }

        let Some(base64_image_data) = self.load_avatar_image(&from, &metadata.checksum).await? else {
            return Ok(None)
        };

        let img = image::load_from_memory(&AvatarMetadata::decode_base64_data(base64_image_data)?)?
            .thumbnail(MAX_IMAGE_DIMENSIONS.0, MAX_IMAGE_DIMENSIONS.1);
        self.avatar_cache
            .cache_avatar_image(&from, img, &metadata)
            .map(Some)
    }

    pub async fn load_and_cache_roster(&self) -> anyhow::Result<Vec<RosterItem>> {
        if let Some(roster_items) = self.data_cache.load_roster_items()? {
            info!("Found cached roster items");
            return Ok(roster_items);
        }

        let roster_items = self.load_roster().await?;
        self.data_cache
            .insert_roster_items(roster_items.as_slice())
            .ok();

        Ok(roster_items)
    }
}

impl<D: DataCache, A: AvatarCache> ClientContext<D, A> {
    #[with_xmpp_client]
    pub async fn load_roster(xmpp: &XMPPClient) -> anyhow::Result<Vec<RosterItem>> {
        let items = xmpp.roster.load_roster(&xmpp.client.context()).await?;
        Ok(items
            .iter()
            .filter_map(|item| RosterItem::try_from(item).ok())
            .collect())
    }

    #[with_xmpp_client]
    pub async fn load_vcard(
        xmpp: &XMPPClient,
        from: &BareJid,
    ) -> anyhow::Result<Option<UserProfile>> {
        xmpp.profile
            .load_vcard(&xmpp.client.context(), from.clone())
            .await
            .and_then(|vcard| vcard.as_ref().map(TryInto::try_into).transpose())
    }

    #[with_xmpp_client]
    pub async fn set_vcard(xmpp: &XMPPClient, profile: &UserProfile) -> anyhow::Result<BareJid> {
        xmpp.profile
            .set_vcard(&xmpp.client.context(), profile.try_into()?)
            .await?;
        Ok(xmpp.jid.clone())
    }

    #[with_xmpp_client]
    pub async fn publish_vcard(xmpp: &XMPPClient, profile: &UserProfile) -> anyhow::Result<()> {
        xmpp.profile
            .publish_vcard(&xmpp.client.context(), profile.try_into()?)
            .await
    }

    #[with_xmpp_client]
    async fn load_avatar_image(
        xmpp: &XMPPClient,
        from: &BareJid,
        image_id: &ImageId,
    ) -> anyhow::Result<Option<String>> {
        xmpp.profile
            .load_avatar_image(&xmpp.client.context(), from.clone(), image_id)
            .await
    }

    #[with_xmpp_client]
    pub async fn set_avatar_image(
        xmpp: &XMPPClient,
        checksum: &ImageId,
        image_data: &[u8],
    ) -> anyhow::Result<()> {
        xmpp.profile
            .set_avatar_image(
                &xmpp.client.context(),
                checksum,
                AvatarMetadata::encode_image_data(image_data),
            )
            .await
    }

    #[with_xmpp_client]
    pub async fn load_latest_avatar_metadata(
        xmpp: &XMPPClient,
        from: &BareJid,
    ) -> anyhow::Result<Option<AvatarMetadata>> {
        xmpp.profile
            .load_latest_avatar_metadata(&xmpp.client.context(), from.clone())
            .await
            .and_then(|info| info.map(TryInto::try_into).transpose())
    }

    #[with_xmpp_client]
    pub async fn set_avatar_metadata(
        xmpp: &XMPPClient,
        bytes_len: usize,
        checksum: &ImageId,
        width: u32,
        height: u32,
    ) -> anyhow::Result<BareJid> {
        xmpp.profile
            .set_avatar_metadata(
                &xmpp.client.context(),
                bytes_len,
                checksum,
                IMAGE_OUTPUT_MIME_TYPE,
                width,
                height,
            )
            .await?;
        Ok(xmpp.jid.clone())
    }

    #[with_xmpp_client]
    pub async fn load_messages_in_chat(
        xmpp: &XMPPClient,
        jid: &BareJid,
        before: impl Into<Option<&message::StanzaId>>,
        after: impl Into<Option<&message::StanzaId>>,
        max_count: impl Into<Option<u32>>,
    ) -> anyhow::Result<(Vec<ArchivedMessage>, Fin)> {
        let result = xmpp
            .mam
            .load_messages_in_chat(&xmpp.client.context(), jid, before, after, max_count)
            .await?;
        Ok((
            result.0.into_iter().map(|m| m.clone()).collect(),
            result.1.clone(),
        ))
    }

    #[with_xmpp_client]
    pub async fn send_message(
        xmpp: &XMPPClient,
        to: impl Into<Jid>,
        body: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        xmpp.chat
            .send_message(&xmpp.client.context(), to, body, Some(ChatState::Active))
    }

    #[with_xmpp_client]
    pub async fn update_message(
        xmpp: &XMPPClient,
        id: message::Id,
        to: impl Into<Jid>,
        body: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        xmpp.chat
            .update_message(&xmpp.client.context(), id, to, body)
    }

    #[with_xmpp_client]
    pub async fn send_chat_state(
        xmpp: &XMPPClient,
        to: impl Into<Jid>,
        chat_state: message::ChatState,
    ) -> anyhow::Result<()> {
        xmpp.chat
            .send_chat_state(&xmpp.client.context(), to, chat_state)
    }

    #[with_xmpp_client]
    pub async fn react_to_message(
        xmpp: &XMPPClient,
        id: message::Id,
        to: impl Into<Jid>,
        reactions: impl IntoIterator<Item = message::Emoji>,
    ) -> anyhow::Result<()> {
        xmpp.chat
            .react_to_message(&xmpp.client.context(), id, to, reactions)
    }

    #[with_xmpp_client]
    pub async fn retract_message(
        xmpp: &XMPPClient,
        id: message::Id,
        to: impl Into<Jid>,
    ) -> anyhow::Result<()> {
        xmpp.chat.retract_message(&xmpp.client.context(), id, to)
    }

    #[with_xmpp_client]
    pub async fn query_server_features(xmpp: &XMPPClient) -> anyhow::Result<()> {
        xmpp.caps
            .query_server_features(&xmpp.client.context())
            .await
    }
}
