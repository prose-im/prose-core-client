// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use xmpp_parsers::mam::Fin;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::stanza::message::mam::ArchivedMessage;

use crate::domain::messaging::models::StanzaId;
use crate::domain::shared::models::RoomType;

#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait MessageArchiveService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_messages(
        &self,
        room_jid: &BareJid,
        room_type: &RoomType,
        before: Option<&StanzaId>,
        after: Option<&StanzaId>,
    ) -> Result<(Vec<ArchivedMessage>, Fin)>;
}
