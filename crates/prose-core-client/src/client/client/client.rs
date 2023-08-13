// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use anyhow::Result;
use jid::FullJid;
use strum_macros::Display;
use tracing::instrument;

use prose_xmpp::mods::{Caps, Chat, Status};
use prose_xmpp::ConnectionError;
use prose_xmpp::{Client as XMPPClient, TimeProvider};

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::{AccountSettings, Availability, Capabilities};
use crate::ClientDelegate;

#[derive(Debug, thiserror::Error, Display)]
pub enum ClientError {
    NotConnected,
}

#[derive(Clone)]
pub struct Client<D: DataCache + 'static, A: AvatarCache + 'static> {
    pub(super) client: XMPPClient,
    pub(super) inner: Arc<ClientInner<D, A>>,
}

pub(super) struct ClientInner<D: DataCache + 'static, A: AvatarCache + 'static> {
    pub caps: Capabilities,
    pub data_cache: D,
    pub avatar_cache: A,
    pub time_provider: Arc<dyn TimeProvider>,
    pub delegate: Option<Box<dyn ClientDelegate<D, A>>>,
}

impl<D: DataCache, A: AvatarCache> Debug for Client<D, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client")
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub fn connected_jid(&self) -> Result<FullJid> {
        self.client
            .connected_jid()
            .ok_or(ClientError::NotConnected.into())
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument(skip(password))]
    pub async fn connect(
        &self,
        jid: &FullJid,
        password: impl AsRef<str>,
        availability: Availability,
    ) -> Result<(), ConnectionError> {
        self.client.connect(jid, password).await?;

        let caps = self.client.get_mod::<Caps>();
        // Send caps before the configured availability since that would otherwise override it
        caps.publish_capabilities((&self.inner.caps).into())
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        let show: xmpp_parsers::presence::Show =
            availability
                .try_into()
                .map_err(|err: anyhow::Error| ConnectionError::Generic {
                    msg: err.to_string(),
                })?;

        let status_mod = self.client.get_mod::<Status>();
        status_mod
            .send_presence(Some(show), None)
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        let chat = self.client.get_mod::<Chat>();
        chat.set_message_carbons_enabled(true)
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        Ok(())
    }

    pub async fn disconnect(&self) {
        self.client.disconnect()
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub async fn delete_cached_data(&self) -> Result<()> {
        self.inner.data_cache.delete_all().await?;
        self.inner.avatar_cache.delete_all_cached_images().await?;
        Ok(())
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub async fn load_account_settings(&self) -> Result<AccountSettings> {
        Ok(self
            .inner
            .data_cache
            .load_account_settings()
            .await?
            .unwrap_or_default())
    }

    pub async fn save_account_settings(&self, settings: &AccountSettings) -> Result<()> {
        self.inner
            .data_cache
            .save_account_settings(settings)
            .await?;
        Ok(())
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub async fn query_server_features(&self) -> Result<()> {
        let caps = self.client.get_mod::<Caps>();
        caps.query_server_features().await
    }
}
