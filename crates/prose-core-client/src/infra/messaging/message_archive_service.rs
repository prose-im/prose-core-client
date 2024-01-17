// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::BareJid;
use xmpp_parsers::mam::Fin;

use prose_xmpp::mods;
use prose_xmpp::stanza::message::mam::ArchivedMessage;
use prose_xmpp::stanza::message::stanza_id;

use crate::domain::messaging::models::StanzaId;
use crate::domain::messaging::services::MessageArchiveService;
use crate::domain::shared::models::RoomType;
use crate::infra::xmpp::XMPPClient;

const MESSAGE_PAGE_SIZE: u32 = 50;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl MessageArchiveService for XMPPClient {
    async fn load_messages(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        before: Option<&StanzaId>,
        after: Option<&StanzaId>,
    ) -> anyhow::Result<(Vec<ArchivedMessage>, Fin)> {
        let mam = self.client.get_mod::<mods::MAM>();
        let before: Option<stanza_id::Id> = before.map(|id| id.as_ref().into());
        let after: Option<stanza_id::Id> = after.map(|id| id.as_ref().into());

        let result = match room_type {
            RoomType::Unknown => unreachable!("Tried to load messages for a pending room"),
            RoomType::DirectMessage => {
                mam.load_messages_in_chat(
                    room_jid,
                    before.as_ref(),
                    after.as_ref(),
                    Some(MESSAGE_PAGE_SIZE as usize),
                )
                .await?
            }
            RoomType::Group
            | RoomType::PrivateChannel
            | RoomType::PublicChannel
            | RoomType::Generic => {
                mam.load_messages_in_muc_chat(
                    room_jid,
                    before.as_ref(),
                    after.as_ref(),
                    Some(MESSAGE_PAGE_SIZE as usize),
                )
                .await?
            }
        };
        Ok(result)
    }
}
