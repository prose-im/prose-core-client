// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use secrecy::Secret;

use crate::app::deps::DynAppContext;
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::ConnectionError;

use crate::client_builder::{
    ClientBuilder, UndefinedAvatarRepository, UndefinedEncryptionService, UndefinedStore,
};
use crate::domain::shared::models::UserId;
use crate::dtos::UserResourceId;
use crate::services::{
    AccountService, BlockListService, CacheService, ConnectionService, ContactListService,
    PreviewService, RoomsService, SidebarService, UploadService, UserDataService,
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
    pub fn builder(
    ) -> ClientBuilder<UndefinedStore, UndefinedAvatarRepository, UndefinedEncryptionService> {
        ClientBuilder::new()
    }
}

pub struct ClientInner {
    pub account: AccountService,
    pub block_list: BlockListService,
    pub cache: CacheService,
    pub contact_list: ContactListService,
    pub(crate) ctx: DynAppContext,
    #[cfg(feature = "debug")]
    pub debug: crate::services::DebugService,
    pub preview: PreviewService,
    pub rooms: RoomsService,
    pub sidebar: SidebarService,
    pub uploads: UploadService,
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
        password: Secret<String>,
    ) -> Result<(), ConnectionError> {
        self.connection.connect(id, password).await
    }

    pub async fn disconnect(&self) {
        self.connection.disconnect().await
    }

    pub fn connected_user_id(&self) -> Option<UserResourceId> {
        self.ctx.connected_id().ok()
    }
}
