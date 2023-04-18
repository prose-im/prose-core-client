use std::fmt::{Debug, Formatter};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use image::GenericImageView;
use jid::{BareJid, FullJid, Jid};
use microtype::Microtype;
use strum_macros::Display;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use prose_core_domain::{Contact, Emoji, Message, MessageId};
use prose_core_lib::modules::profile::avatar::ImageId;
use prose_core_lib::modules::{ArchivedMessage, Caps, Chat, Fin, Profile, Roster, MAM};
use prose_core_lib::stanza::{message, Namespace};
use prose_core_lib::{Connection, ConnectionError, ConnectionEvent};

use crate::cache::{
    AvatarCache, DataCache, IMAGE_OUTPUT_FORMAT, IMAGE_OUTPUT_MIME_TYPE, MAX_IMAGE_DIMENSIONS,
};
use crate::client::{ClientContext, ClientEvent, ModuleDelegate, XMPPClient};
use crate::domain_ext::{ChatState, MessageExt};
use crate::types::{
    AvatarMetadata, Capabilities, Feature, MessageLike, Page, RosterItem, UserProfile,
};
use crate::{domain_ext, ClientDelegate};

#[derive(Debug, thiserror::Error, Display)]
pub enum ClientError {
    NotConnected,
}

pub struct Client<D: DataCache + 'static, A: AvatarCache + 'static> {
    ctx: Arc<ClientContext<D, A>>,
}

impl<D: DataCache, A: AvatarCache> Debug for Client<D, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client")
    }
}

const MESSAGE_PAGE_SIZE: u32 = 50;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub fn new(data_cache: D, avatar_cache: A, delegate: Option<Box<dyn ClientDelegate>>) -> Self {
        let capabilities = Capabilities::new(
            "Prose",
            "https://www.prose.org",
            vec![
                Feature::new(Namespace::AvatarData, false),
                Feature::new(Namespace::AvatarMetadata, false),
                Feature::new(Namespace::AvatarMetadata, true),
                Feature::new(Namespace::Ping, false),
                Feature::new(Namespace::PubSub, false),
                Feature::new(Namespace::PubSub, true),
                Feature::new(Namespace::Receipts, false),
                Feature::new(Namespace::VCard, false),
                Feature::new(Namespace::VCard, true),
            ],
        );

        let ctx = ClientContext {
            capabilities,
            xmpp: RwLock::new(None),
            delegate: delegate.map(Arc::new),
            data_cache,
            avatar_cache,
        };

        Client { ctx: Arc::new(ctx) }
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument(skip(password))]
    pub async fn connect(
        &self,
        jid: &FullJid,
        password: impl Into<String> + Debug,
    ) -> anyhow::Result<(), ConnectionError> {
        if let Some(xmpp) = self.ctx.xmpp.write().await.take() {
            xmpp.client.disconnect();
        }

        let module_delegate = Arc::new(ModuleDelegate::new(self.ctx.clone()));

        let chat = Arc::new(Chat::new(Some(module_delegate.clone())));
        let roster = Arc::new(Roster::new());
        let mam = Arc::new(MAM::new());
        let profile = Arc::new(Profile::new(Some(module_delegate.clone())));
        let caps = Arc::new(Caps::new(Some(module_delegate)));

        let connection_handler: Box<dyn FnMut(&dyn Connection, &ConnectionEvent) + Send> =
            match &self.ctx.delegate {
                Some(delegate) => {
                    let delegate = delegate.clone();
                    Box::new(move |_, event| {
                        delegate.handle_event(ClientEvent::ConnectionStatusChanged {
                            event: event.clone(),
                        })
                    })
                }
                None => Box::new(|_, _| {}),
            };

        let connected_client = prose_core_lib::Client::new()
            .register_module(chat.clone())
            .register_module(roster.clone())
            .register_module(mam.clone())
            .register_module(profile.clone())
            .register_module(caps.clone())
            .set_connection_handler(connection_handler)
            .connect(jid, password)
            .await?;

        chat.set_message_carbons_enabled(&connected_client.context(), true)
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        caps.publish_capabilities(&connected_client.context(), (&self.ctx.capabilities).into())
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        let xmpp = XMPPClient {
            jid: BareJid::from(jid.clone()),
            client: connected_client,
            roster,
            profile,
            chat,
            mam,
            caps,
        };

        *self.ctx.xmpp.write().await = Some(xmpp);
        Ok(())
    }

    pub async fn disconnect(&self) {
        if let Some(xmpp) = self.ctx.xmpp.write().await.take() {
            xmpp.client.disconnect();
        }
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub async fn delete_cached_data(&self) -> anyhow::Result<()> {
        self.ctx.data_cache.delete_all()?;
        self.ctx.avatar_cache.delete_all_cached_images()?;
        Ok(())
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    pub async fn load_roster(&self) -> anyhow::Result<Vec<RosterItem>> {
        self.ctx.load_and_cache_roster().await
    }

    #[instrument]
    pub async fn load_profile(
        &self,
        from: impl Into<BareJid> + Debug,
    ) -> anyhow::Result<UserProfile> {
        let from = from.into();

        if let Some(cached_profile) = self.ctx.data_cache.load_user_profile(&from)? {
            info!("Found cached profile for {}", from);
            return Ok(cached_profile);
        }

        let Some(profile) = self.ctx.load_vcard(&from).await? else {
            return Ok(UserProfile::default())
        };

        self.ctx.data_cache.insert_user_profile(&from, &profile)?;
        Ok(profile.into_inner())
    }

    #[instrument]
    pub async fn save_profile(&self, profile: UserProfile) -> anyhow::Result<()> {
        let profile: domain_ext::UserProfile = profile.into();
        let jid = self.ctx.set_vcard(&profile).await?;
        self.ctx.publish_vcard(&profile).await?;

        self.ctx.data_cache.insert_user_profile(&jid, &profile)?;

        Ok(())
    }

    #[instrument]
    pub async fn load_avatar(
        &self,
        from: impl Into<Jid> + Debug,
    ) -> anyhow::Result<Option<PathBuf>> {
        let jid = BareJid::from(from.into());

        let metadata = match self.ctx.data_cache.load_avatar_metadata(&jid)? {
            Some(md) => {
                info!("Found cached metadata for {}", jid);
                Ok::<_, anyhow::Error>(Some(md))
            }
            None => {
                let Some(metadata) = self.ctx.load_latest_avatar_metadata(&jid).await? else {
                    return Ok(None)
                };
                self.ctx
                    .data_cache
                    .insert_avatar_metadata(&jid, &metadata)?;
                Ok(Some(metadata))
            }
        }?;

        let Some(metadata) = metadata else {
            return Ok(None)
        };

        self.ctx.load_and_cache_avatar_image(&jid, &metadata).await
    }

    #[instrument]
    pub async fn save_avatar(&self, image_path: &Path) -> anyhow::Result<PathBuf> {
        let now = Instant::now();
        info!("Opening & resizing image at {:?}…", image_path);

        let img =
            image::open(image_path)?.thumbnail(MAX_IMAGE_DIMENSIONS.0, MAX_IMAGE_DIMENSIONS.1);
        info!(
            "Opening image & resizing finished after {:.2?}",
            now.elapsed()
        );

        let mut image_data = Vec::new();
        img.write_to(&mut Cursor::new(&mut image_data), IMAGE_OUTPUT_FORMAT)?;

        let metadata = AvatarMetadata::new(
            IMAGE_OUTPUT_MIME_TYPE,
            AvatarMetadata::generate_sha1_checksum(&image_data).into(),
            img.dimensions().0,
            img.dimensions().1,
        );

        info!("Uploading avatar…");
        self.ctx
            .set_avatar_image(&metadata.checksum, &image_data)
            .await?;

        info!("Uploading avatar metadata…");
        let jid = self
            .ctx
            .set_avatar_metadata(
                image_data.len(),
                &metadata.checksum,
                metadata.width,
                metadata.height,
            )
            .await?;

        info!("Caching avatar metadata");
        self.ctx
            .data_cache
            .insert_avatar_metadata(&jid, &metadata)?;

        info!("Caching image locally…");
        let path = self
            .ctx
            .avatar_cache
            .cache_avatar_image(&jid, img, &metadata)?;

        Ok(path)
    }

    #[instrument]
    pub async fn load_contacts(&self) -> anyhow::Result<Vec<Contact>> {
        if !self.ctx.data_cache.has_valid_roster_items()? {
            self.ctx.load_and_cache_roster().await?;
        }

        let contacts: Vec<(Contact, Option<ImageId>)> = self.ctx.data_cache.load_contacts()?;

        Ok(contacts
            .into_iter()
            .map(|(mut contact, image_id)| {
                if let Some(image_id) = image_id {
                    contact.avatar = self
                        .ctx
                        .avatar_cache
                        .cached_avatar_image_url(&contact.jid, &image_id)
                }
                contact
            })
            .collect())
    }

    #[instrument]
    pub async fn load_latest_messages(
        &self,
        from: &BareJid,
        since: impl Into<Option<&MessageId>> + Debug,
        load_from_server: bool,
    ) -> anyhow::Result<Vec<Message>> {
        // TODO: See comment below
        // It's possible that newly loaded messages affect already visible ones in the client. In
        // this case we'll need to generate the appropriate `ClientEvent`s.

        // TODO: See comment below
        // It might also be possible that we do not receive the absolute last message from the
        // server if more than MESSAGE_PAGE_SIZE messages were sent since the last message we've
        // seen. In that case we need to compare the fin element's last id with the stanza id of
        // the last message to see if we've received it. Otherwise we'll need

        let since: Option<message::Id> = since.into().map(|id| id.as_ref().into());

        let mut messages = if let Some(since) = &since {
            info!(
                "Loading messages in conversation {} after {} from local cache…",
                from, since
            );
            self.ctx
                .data_cache
                .load_messages_after(from, since, Some(MESSAGE_PAGE_SIZE))?
        } else {
            info!(
                "Loading last page of messages in conversation {} from local cache…",
                from
            );
            self.ctx
                .data_cache
                .load_messages_before(from, None, MESSAGE_PAGE_SIZE)?
                .map(|page| page.items)
                .unwrap_or_else(|| vec![])
        };

        info!("Found {} messages in local cache.", messages.len());

        // We take either the stanza_id of the last cached message or the first stanza_id that is
        // followed by a local message for which we don't know the stanza_id yet. This way we're
        // syncing up with the server.
        let stanza_id = 'outer: loop {
            for (l, r) in messages.iter().zip(messages.iter().skip(1)) {
                if let (Some(l), None) = (&l.stanza_id, &r.stanza_id) {
                    break 'outer Some(l);
                }
            }
            break messages.last().and_then(|m| m.stanza_id.as_ref());
        };

        if load_from_server {
            info!("Loading messages from server since {:?}…", stanza_id);
            let mut remote_messages = self
                .ctx
                .load_messages_in_chat(from, None, stanza_id, MESSAGE_PAGE_SIZE)
                .await?
                .0
                .iter()
                .map(|msg| MessageLike::try_from(msg))
                .collect::<Result<Vec<_>, _>>()?;

            info!("Found {} messages. Saving to cache…", remote_messages.len());
            self.ctx
                .data_cache
                .insert_messages(remote_messages.iter())?;

            // Remove all messages from the tail of the local messages including the message that
            // matches the first message returned from the server so that we don't have any
            // duplicates but the latest remote data in our vec.
            //
            // Local Remote
            //   1
            //   2
            //   3     3
            //   4     4
            //   5     5

            if let Some(first_remote_message_id) = remote_messages.first().map(|m| &m.id) {
                let cutoff_idx = messages.iter().rev().enumerate().find_map(|(idx, msg)| {
                    if &msg.id == first_remote_message_id {
                        Some(messages.len() - idx - 1)
                    } else {
                        None
                    }
                });

                if let Some(cutoff_idx) = cutoff_idx {
                    debug!(
                        "Truncating local messages to messages before {:?} at index {:?}",
                        first_remote_message_id, cutoff_idx
                    );
                    messages.truncate(cutoff_idx);
                } else {
                    debug!("Couldn't find the first remote message in our set of local messages. Discarding all local messages.");
                    messages.retain(|_| false)
                }
            }

            messages.append(&mut remote_messages);
        } else {
            info!("Skipping server round trip.")
        }

        Ok(Message::reducing_messages(messages))
    }

    #[instrument]
    pub async fn load_messages_before(
        &self,
        from: &BareJid,
        before: impl Into<&MessageId> + Debug,
    ) -> anyhow::Result<Page<Message>> {
        // TODO: See comment below
        // It might be possible that we have a holes in our cached messages, if we've synced with
        // the server only sporadically or in busy conversations. Our cache would still happily
        // return a page and report success since it found some messages. Do we always need a
        // server round trip?
        //
        // Local Remote
        //         1
        //   2     2
        //         3
        //         4
        //   5     5
        //   6     6

        let before: message::Id = before.into().as_ref().into();

        // If we have messages cached already return these without a round trip to the server…
        if let Some(cached_messages) =
            self.ctx
                .data_cache
                .load_messages_before(from, Some(&before), MESSAGE_PAGE_SIZE)?
        {
            info!("Returning cached messages for conversation {}…", from);
            return self.enriching_messages_from_cache(from, cached_messages);
        }

        // We couldn't find any older messages but we need to have the one matching the id at least.
        // So we'll fetch that to translate the MessageId into a StanzaId for the server.
        let Some(stanza_id) = self.ctx.data_cache.load_stanza_id(from, &before)? else {
            return Err(anyhow::format_err!("Could not determine stanza_id for message with id {}", before));
        };

        info!("Loading messages for conversation {}…", from);
        let (messages, fin): (Vec<ArchivedMessage>, Fin) = self
            .ctx
            .load_messages_in_chat(from, &stanza_id, None, MESSAGE_PAGE_SIZE)
            .await?;

        let Some(first_message) = messages.first() else {
            return Ok(Page {
                items: vec![],
                is_complete: true
            })
        };

        let oldest_message_id: Option<message::Id> = if fin.is_complete() {
            first_message.message.message().and_then(|m| m.id())
        } else {
            None
        };

        let parsed_messages = messages
            .iter()
            .map(|msg| match MessageLike::try_from(msg) {
                Ok(mut msg) => {
                    msg.is_first_message = Some(&msg.id) == oldest_message_id.as_ref();
                    Ok(msg)
                }
                Err(err) => Err(err),
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.ctx
            .data_cache
            .insert_messages(parsed_messages.iter())?;

        self.enriching_messages_from_cache(
            from,
            Page {
                items: parsed_messages,
                is_complete: fin.is_complete(),
            },
        )
    }

    #[instrument]
    pub async fn load_messages_with_ids(
        &self,
        conversation: &BareJid,
        ids: &[MessageId],
    ) -> anyhow::Result<Vec<Message>> {
        let ids = ids
            .iter()
            .map(|id| id.as_ref().into())
            .collect::<Vec<message::Id>>();
        let messages = self.ctx.data_cache.load_messages_targeting(
            conversation,
            ids.as_slice(),
            None,
            true,
        )?;
        debug!(
            "{}",
            messages
                .iter()
                .map(|m| format!("{:?}", m))
                .collect::<Vec<_>>()
                .join("\n")
        );
        Ok(Message::reducing_messages(messages))
    }

    #[instrument]
    pub async fn send_message(
        &self,
        to: impl Into<Jid> + Debug,
        body: impl AsRef<str> + Debug,
    ) -> anyhow::Result<()> {
        self.ctx.send_message(to, body).await
    }

    #[instrument]
    pub async fn update_message(
        &self,
        conversation: impl Into<Jid> + Debug,
        id: MessageId,
        body: impl AsRef<str> + Debug,
    ) -> anyhow::Result<()> {
        self.ctx
            .update_message(id.into_inner().into(), conversation, body)
            .await?;
        Ok(())
    }

    #[instrument]
    pub async fn send_chat_state(
        &self,
        conversation: impl Into<Jid> + Debug,
        chat_state: prose_core_domain::ChatState,
    ) -> anyhow::Result<()> {
        self.ctx
            .send_chat_state(conversation, ChatState(chat_state).into())
            .await?;
        Ok(())
    }

    #[instrument]
    pub async fn toggle_reaction_to_message(
        &self,
        conversation: impl Into<Jid> + Debug,
        id: MessageId,
        emoji: Emoji,
    ) -> anyhow::Result<()> {
        let opt = &(*self.ctx.xmpp.read().await);
        let xmpp = opt.as_ref().ok_or(ClientError::NotConnected)?;

        let current_user = &xmpp.jid;
        let conversation = BareJid::from(conversation.into());
        let message_id = message::Id::from(id.into_inner());
        let message = self.load_message(&conversation, &message_id).await?;
        let mut emoji_found = false;

        let mut reactions = message
            .reactions
            .into_iter()
            .filter_map(|r| {
                if r.from.contains(current_user) {
                    if r.emoji == emoji {
                        emoji_found = true;
                        return None;
                    }
                    Some(prose_core_lib::stanza::message::Emoji::from(
                        r.emoji.into_inner(),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if !emoji_found {
            reactions.push(prose_core_lib::stanza::message::Emoji::from(
                emoji.into_inner(),
            ))
        }

        self.ctx
            .react_to_message(message_id, conversation, reactions)
            .await?;
        Ok(())
    }

    #[instrument]
    pub async fn retract_message(
        &self,
        conversation: impl Into<Jid> + Debug,
        id: MessageId,
    ) -> anyhow::Result<()> {
        self.ctx
            .retract_message(id.into_inner().into(), conversation)
            .await?;
        Ok(())
    }

    #[instrument]
    // Signature is incomplete. send_presence(&self, show: Option<ShowKind>, status: &Option<String>)
    pub fn send_presence(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn query_server_features(&self) -> anyhow::Result<()> {
        self.ctx.query_server_features().await
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    /// Takes a `Page` of `MessageLike` objects, fetches all `MessageLike` objects from the cache
    /// that modify messages in `page` and returns the reduced `Message`s.
    ///
    /// # Arguments
    ///
    /// * `conversation`: The conversation to which `page` belongs.
    /// * `page`: A page of messages.
    fn enriching_messages_from_cache(
        &self,
        conversation: &BareJid,
        page: Page<MessageLike>,
    ) -> anyhow::Result<Page<Message>> {
        let message_ids = page.items.iter().map(|m| m.id.clone()).collect::<Vec<_>>();
        let last_message_id = &page.items.last().unwrap().id;
        let modifiers = self.ctx.data_cache.load_messages_targeting(
            &conversation,
            &message_ids,
            last_message_id,
            false,
        )?;

        let reduced_messages =
            Message::reducing_messages(page.items.into_iter().chain(modifiers.into_iter()));

        Ok(Page {
            items: reduced_messages,
            is_complete: page.is_complete,
        })
    }

    async fn load_message(
        &self,
        conversation: &BareJid,
        message_id: &message::Id,
    ) -> anyhow::Result<Message> {
        let ids = [MessageId::from(message_id.as_ref())];
        self.load_messages_with_ids(conversation, &ids)
            .await?
            .pop()
            .ok_or(anyhow::format_err!("No message with id {}", ids[0]))
    }
}
