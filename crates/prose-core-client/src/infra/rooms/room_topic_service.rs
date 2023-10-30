// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use prose_xmpp::mods;

use crate::domain::rooms::services::RoomTopicService;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl RoomTopicService for XMPPClient {
    async fn set_topic(&self, room_jid: &BareJid, subject: Option<&str>) -> Result<()> {
        let muc = self.client.get_mod::<mods::MUC>();
        muc.set_room_subject(room_jid, subject).await
    }
}
