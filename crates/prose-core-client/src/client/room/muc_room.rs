use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::room::{Generic, Group, PrivateChannel, PublicChannel, Room, MESSAGE_PAGE_SIZE};
use crate::types::{Message, MessageId, MessageLike};
use anyhow::Result;
use prose_xmpp::mods;
use tracing::debug;

pub trait MUCRoom {}

impl MUCRoom for Group {}
impl MUCRoom for PrivateChannel {}
impl MUCRoom for PublicChannel {}
impl MUCRoom for Generic {}

impl<Kind, D: DataCache, A: AvatarCache> Room<Kind, D, A>
where
    Kind: MUCRoom,
{
    pub async fn load_latest_messages(
        &self,
        _since: impl Into<Option<&MessageId>>,
        _load_from_server: bool,
    ) -> Result<Vec<Message>> {
        debug!("Loading muc messages from server…");

        let mam = self.inner.xmpp.get_mod::<mods::MAM>();
        let result = mam
            .load_messages_in_muc_chat(
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
