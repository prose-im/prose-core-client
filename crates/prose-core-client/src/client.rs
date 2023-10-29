// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use jid::BareJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::ConnectionError;

use crate::app::services::{
    AccountService, ConnectionService, ContactsService, RoomsService, UserDataService,
};
use crate::client_builder::{ClientBuilder, UndefinedAvatarCache, UndefinedStore};
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
    pub(crate) connection: ConnectionService,
    pub account: AccountService,
    pub contacts: ContactsService,
    pub rooms: RoomsService,
    pub user_data: UserDataService,
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
        jid: &BareJid,
        password: impl AsRef<str>,
    ) -> Result<(), ConnectionError> {
        self.connection.connect(jid, password).await
    }

    pub async fn disconnect(&self) {
        self.connection.disconnect().await
    }
}
