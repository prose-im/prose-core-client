// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::Room;
use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::room::MESSAGE_PAGE_SIZE;
use crate::types::{Message, MessageId, MessageLike};
use anyhow::Result;
use prose_xmpp::mods;
use tracing::debug;

pub struct DirectMessage;

impl<D: DataCache, A: AvatarCache> Room<DirectMessage, D, A> {
    pub async fn load_latest_messages(
        &self,
        _since: impl Into<Option<&MessageId>>,
        _load_from_server: bool,
    ) -> Result<Vec<Message>> {
        debug!("Loading messages from server…");

        let mam = self.inner.xmpp.get_mod::<mods::MAM>();
        let result = mam
            .load_messages_in_chat(
                &self.inner.jid,
                None,
                None,
                Some(MESSAGE_PAGE_SIZE as usize),
            )
            .await?;

        let messages = result
            .0
            .iter()
            .map(|msg| MessageLike::try_from(msg))
            .collect::<Result<Vec<_>, _>>()?;

        debug!("Found {} messages. Saving to cache…", messages.len());
        self.inner
            .client
            .data_cache
            .insert_messages(messages.iter())
            .await?;

        Ok(Message::reducing_messages(messages))
    }
}
