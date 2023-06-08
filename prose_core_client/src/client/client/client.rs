use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use jid::{BareJid, FullJid};
use strum_macros::Display;
use tokio::sync::RwLock;
use tracing::instrument;

use prose_core_domain::Availability;
use prose_core_lib::modules::{Caps, Chat, Profile, Roster, MAM};
use prose_core_lib::stanza::{presence, Namespace};
use prose_core_lib::{Connection, ConnectionError, ConnectionEvent, IDProvider, TimeProvider};

use crate::cache::{AvatarCache, DataCache};
use crate::client::{ClientContext, ClientEvent, ModuleDelegate, XMPPClient};
use crate::types::{AccountSettings, Capabilities, Feature};
use crate::ClientDelegate;

pub type ConnectorProvider = Box<dyn Fn() -> Box<dyn prose_core_lib::Connector> + Send + Sync>;

#[derive(Debug, thiserror::Error, Display)]
pub enum ClientError {
    NotConnected,
}

pub struct Client<D: DataCache + 'static, A: AvatarCache + 'static> {
    connector_provider: ConnectorProvider,
    id_provider: Arc<dyn IDProvider>,
    time_provider: Arc<dyn TimeProvider>,
    pub(crate) ctx: Arc<ClientContext<D, A>>,
}

impl<D: DataCache, A: AvatarCache> Debug for Client<D, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client")
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub fn new(
        connector_provider: ConnectorProvider,
        data_cache: D,
        avatar_cache: A,
        delegate: Option<Box<dyn ClientDelegate>>,
        id_provider: Arc<dyn IDProvider>,
        time_provider: Arc<dyn TimeProvider>,
    ) -> Self {
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

        Client {
            connector_provider,
            ctx: Arc::new(ctx),
            id_provider,
            time_provider,
        }
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument(skip(password))]
    pub async fn connect(
        &self,
        jid: &FullJid,
        password: impl Into<String> + Debug,
        availability: Availability,
        status: Option<&str>,
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
            .set_connector((self.connector_provider)())
            .set_connection_handler(connection_handler)
            .set_id_provider(self.id_provider.clone())
            .set_time_provider(self.time_provider.clone())
            .connect(jid, password)
            .await?;

        // Send caps before the configured availability since that would otherwise override it
        caps.publish_capabilities(&connected_client.context(), (&self.ctx.capabilities).into())
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        let show: presence::Show = crate::domain_ext::Availability::from(availability)
            .try_into()
            .map_err(|err: anyhow::Error| ConnectionError::Generic {
                msg: err.to_string(),
            })?;
        profile
            .send_presence(&connected_client.context(), Some(show), status)
            .await
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        chat.set_message_carbons_enabled(&connected_client.context(), true)
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
    pub async fn load_account_settings(&self) -> anyhow::Result<AccountSettings> {
        Ok(self
            .ctx
            .data_cache
            .load_account_settings()?
            .unwrap_or_default())
    }

    pub async fn save_account_settings(&self, settings: &AccountSettings) -> anyhow::Result<()> {
        self.ctx.data_cache.save_account_settings(settings)
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub async fn query_server_features(&self) -> anyhow::Result<()> {
        self.ctx.query_server_features().await
    }
}
