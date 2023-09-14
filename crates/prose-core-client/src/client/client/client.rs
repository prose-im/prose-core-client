// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use anyhow::Result;
use jid::{BareJid, FullJid, Jid};
use parking_lot::RwLock;
use strum_macros::Display;
use tracing::{error, instrument};

use prose_xmpp::mods::{Chat, Status};
use prose_xmpp::{mods, ConnectionError};
use prose_xmpp::{Client as XMPPClient, TimeProvider};

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::muc::Room;
use crate::types::{muc, Bookmarks};
use crate::types::{AccountSettings, Availability, Capabilities, SoftwareVersion};
use crate::util::PresenceMap;
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
    pub software_version: SoftwareVersion,
    pub delegate: Option<Box<dyn ClientDelegate<D, A>>>,
    pub presences: RwLock<PresenceMap>,
    pub muc_service: RwLock<Option<muc::Service>>,
    pub bookmarks: RwLock<Bookmarks>,
    pub connected_rooms: RwLock<HashMap<BareJid, Room>>,
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

        let show: xmpp_parsers::presence::Show =
            availability
                .try_into()
                .map_err(|err: anyhow::Error| ConnectionError::Generic {
                    msg: err.to_string(),
                })?;

        let status_mod = self.client.get_mod::<Status>();
        status_mod
            .send_presence(Some(show), None, Some((&self.inner.caps).into()))
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        let chat = self.client.get_mod::<Chat>();
        chat.set_message_carbons_enabled(true)
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        self.gather_server_features()
            .await
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        let client = Client {
            client: self.client.clone(),
            inner: self.inner.clone(),
        };
        prose_xmpp::spawn(async move {
            if let Err(error) = client.perform_post_connect_tasks().await {
                error!("Failed to run post-connect-tasks: {}", error.to_string())
            }
        });

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
    async fn gather_server_features(&self) -> Result<()> {
        let caps = self.client.get_mod::<mods::Caps>();
        let disco_items = caps.query_server_disco_items(None).await?;

        for item in disco_items.items {
            let info = caps.query_disco_info(item.jid.clone(), None).await?;

            if info
                .identities
                .iter()
                .find(|ident| ident.category == "conference")
                .is_none()
            {
                continue;
            }

            *self.inner.muc_service.write() = Some(muc::Service {
                user_jid: self.connected_jid()?.into_bare(),
                client: self.client.clone(),
                jid: item.jid.into_bare(),
            });
            break;
        }

        Ok(())
    }

    async fn perform_post_connect_tasks(&self) -> Result<()> {
        self.load_and_connect_bookmarks().await?;
        Ok(())
    }
}

impl<D: DataCache, A: AvatarCache> ClientInner<D, A> {
    /// Tries to resolve `jid` to a FullJid by appending the available resource with the highest
    /// priority. If no available resource is found, returns `jid` as a `Jid`.
    pub(super) fn resolve_to_full_jid(&self, jid: &BareJid) -> Jid {
        let presences = self.presences.read();
        let Some(resource) = presences
            .get_highest_presence(jid)
            .and_then(|entry| entry.resource.as_deref())
        else {
            return Jid::Bare(jid.clone());
        };
        return jid
            .with_resource_str(resource)
            .map(Jid::Full)
            .unwrap_or(Jid::Bare(jid.clone()));
    }
}
