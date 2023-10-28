// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::FullJid;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::ConnectionError;

use crate::domain::connection::models::ServerFeatures;

#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait ConnectionService: SendUnlessWasm + SyncUnlessWasm {
    async fn connect(&self, jid: &FullJid, password: &str) -> Result<(), ConnectionError>;
    async fn disconnect(&self);

    async fn set_message_carbons_enabled(&self, is_enabled: bool) -> Result<()>;
    async fn load_server_features(&self) -> Result<ServerFeatures>;
}
