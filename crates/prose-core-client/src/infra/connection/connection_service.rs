// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use minidom::Element;

use prose_xmpp::{mods, ConnectionError};

use crate::domain::connection::models::ServerFeatures;
use crate::domain::connection::services::ConnectionService;
use crate::dtos::UserResourceId;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ConnectionService for XMPPClient {
    async fn connect(&self, jid: &UserResourceId, password: &str) -> Result<(), ConnectionError> {
        self.client.connect(jid.as_ref(), password).await
    }

    async fn disconnect(&self) {
        self.client.disconnect()
    }

    async fn set_message_carbons_enabled(&self, is_enabled: bool) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.set_message_carbons_enabled(is_enabled)?;
        Ok(())
    }

    async fn load_server_features(&self) -> Result<ServerFeatures> {
        let caps = self.client.get_mod::<mods::Caps>();
        let disco_items = caps.query_server_disco_items(None).await?;
        let mut server_features = ServerFeatures::default();

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

            server_features.muc_service = Some(item.jid.into_bare());
            break;
        }

        Ok(server_features)
    }

    async fn send_raw_stanza(&self, stanza: Element) -> Result<()> {
        self.client.send_raw_stanza(stanza)
    }
}
