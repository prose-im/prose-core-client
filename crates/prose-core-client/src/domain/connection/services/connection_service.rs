// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use minidom::Element;
use secrecy::Secret;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::ConnectionError;

use crate::domain::connection::models::ServerFeatures;
use crate::domain::shared::models::UserResourceId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait ConnectionService: SendUnlessWasm + SyncUnlessWasm {
    async fn connect(
        &self,
        jid: &UserResourceId,
        password: Secret<String>,
    ) -> Result<(), ConnectionError>;
    async fn disconnect(&self);

    async fn set_message_carbons_enabled(&self, is_enabled: bool) -> Result<()>;
    async fn load_server_features(&self) -> Result<ServerFeatures>;

    async fn send_raw_stanza(&self, stanza: Element) -> Result<()>;
}
