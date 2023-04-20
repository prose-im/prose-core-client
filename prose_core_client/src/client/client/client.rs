use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use jid::{BareJid, FullJid};
use strum_macros::Display;
use tokio::sync::RwLock;
use tracing::instrument;

use prose_core_domain::Contact;
use prose_core_lib::modules::profile::avatar::ImageId;
use prose_core_lib::modules::{Caps, Chat, Profile, Roster, MAM};
use prose_core_lib::stanza::Namespace;
use prose_core_lib::{Connection, ConnectionError, ConnectionEvent};

use crate::cache::{AvatarCache, DataCache};
use crate::client::{CachePolicy, ClientContext, ClientEvent, ModuleDelegate, XMPPClient};
use crate::types::{Capabilities, Feature};
use crate::ClientDelegate;

#[derive(Debug, thiserror::Error, Display)]
pub enum ClientError {
    NotConnected,
}

pub struct Client<D: DataCache + 'static, A: AvatarCache + 'static> {
    pub(super) ctx: Arc<ClientContext<D, A>>,
}

impl<D: DataCache, A: AvatarCache> Debug for Client<D, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client")
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub fn new(data_cache: D, avatar_cache: A, delegate: Option<Box<dyn ClientDelegate>>) -> Self {
        let capabilities = Capabilities::new(
            "Prose",
            "https://www.prose.org",
            vec![
                Feature::new(Namespace::AvatarData, false),
                Feature::new(Namespace::AvatarMetadata, false),
                Feature::new(Namespace::AvatarMetadata, true),
                Feature::new(Namespace::ChatStates, false),
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
    pub async fn load_contacts(&self, cache_policy: CachePolicy) -> anyhow::Result<Vec<Contact>> {
        if cache_policy == CachePolicy::ReloadIgnoringCacheData
            || !self.ctx.data_cache.has_valid_roster_items()?
        {
            if cache_policy == CachePolicy::ReturnCacheDataDontLoad {
                return Ok(vec![]);
            }

            let roster_items = self.ctx.load_roster().await?;
            self.ctx
                .data_cache
                .insert_roster_items(roster_items.as_slice())
                .ok();
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
    // Signature is incomplete. send_presence(&self, show: Option<ShowKind>, status: &Option<String>)
    pub fn send_presence(&self) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn query_server_features(&self) -> anyhow::Result<()> {
        self.ctx.query_server_features().await
    }
}
