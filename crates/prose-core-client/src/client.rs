// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::ConnectionError;

use crate::client_builder::{ClientBuilder, UndefinedAvatarCache, UndefinedStore};
use crate::domain::shared::models::UserId;
use crate::services::{
    AccountService, CacheService, ConnectionService, ContactsService, RoomsService, SidebarService,
    UserDataService,
};
use crate::ClientEvent;

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

pub trait ClientDelegate: SendUnlessWasm + SyncUnlessWasm {
    fn handle_event(&self, client: Client, event: ClientEvent);
}

impl Client {
    pub fn builder() -> ClientBuilder<UndefinedStore, UndefinedAvatarCache> {
        ClientBuilder::new()
    }
}

pub struct ClientInner {
    pub account: AccountService,
    pub cache: CacheService,
    pub contacts: ContactsService,
    #[cfg(feature = "debug")]
    pub debug: crate::services::DebugService,
    pub rooms: RoomsService,
    pub sidebar: SidebarService,
    pub user_data: UserDataService,
    pub(crate) connection: ConnectionService,
}

impl From<Arc<ClientInner>> for Client {
    fn from(inner: Arc<ClientInner>) -> Self {
        Client { inner }
    }
}

impl Deref for Client {
    type Target = ClientInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Client {
    pub async fn connect(
        &self,
        id: &UserId,
        password: impl AsRef<str>,
    ) -> Result<(), ConnectionError> {
        self.connection.connect(id, password).await
    }

    pub async fn disconnect(&self) {
        self.connection.disconnect().await
    }
}

#[cfg(feature = "debug")]
impl Client {
    pub async fn send_raw_stanza(&self, stanza: impl Into<minidom::Element>) -> Result<()> {
        self.inner.connection.send_raw_stanza(stanza.into()).await
    }
}
