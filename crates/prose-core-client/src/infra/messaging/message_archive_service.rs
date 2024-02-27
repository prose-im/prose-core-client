// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use xmpp_parsers::mam::Complete;

use prose_xmpp::mods;
use prose_xmpp::stanza::message::stanza_id;

use crate::domain::messaging::models::StanzaId;
use crate::domain::messaging::services::{MessageArchiveService, MessagePage};
use crate::dtos::RoomId;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl MessageArchiveService for XMPPClient {
    async fn load_messages(
        &self,
        room_id: &RoomId,
        before: Option<&StanzaId>,
        after: Option<&StanzaId>,
        batch_size: u32,
    ) -> Result<MessagePage> {
        let mam = self.client.get_mod::<mods::MAM>();
        let before: Option<stanza_id::Id> = before.map(|id| id.as_ref().into());
        let after: Option<stanza_id::Id> = after.map(|id| id.as_ref().into());

        let (messages, fin) = match room_id {
            RoomId::User(id) => {
                mam.load_messages_in_chat(
                    id.as_ref(),
                    before.as_ref(),
                    after.as_ref(),
                    Some(batch_size as usize),
                )
                .await?
            }
            RoomId::Muc(id) => {
                mam.load_messages_in_muc_chat(
                    id.as_ref(),
                    before.as_ref(),
                    after.as_ref(),
                    Some(batch_size as usize),
                )
                .await?
            }
        };
        Ok(MessagePage {
            messages,
            is_last: fin.complete == Complete::True,
        })
    }
}
