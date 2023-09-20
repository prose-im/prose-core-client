// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Debug;

use anyhow::{format_err, Result};
use jid::{BareJid, Jid};
use tracing::debug;
use xmpp_parsers::mam::Complete;
use xmpp_parsers::message::MessageType;

use prose_xmpp::mods::{Chat, MAM};
use prose_xmpp::stanza::message;
use prose_xmpp::stanza::message::ChatState;

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::{Emoji, Message, MessageId, MessageLike, Page};

use super::Client;

const MESSAGE_PAGE_SIZE: u32 = 50;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[deprecated(note = "Use the Room API instead.")]
    pub async fn load_latest_messages(
        &self,
        from: &BareJid,
        _since: impl Into<Option<&MessageId>> + Debug,
        _load_from_server: bool,
    ) -> Result<Vec<Message>> {
        debug!("Loading messages from server…");

        let mam = self.client.get_mod::<MAM>();
        let result = mam
            .load_messages_in_chat(from, None, None, Some(MESSAGE_PAGE_SIZE as usize))
            .await?;

        let messages = result
            .0
            .iter()
            .map(|msg| MessageLike::try_from(msg))
            .collect::<Result<Vec<_>, _>>()?;

        debug!("Found {} messages. Saving to cache…", messages.len());
        self.inner
            .data_cache
            .insert_messages(messages.iter())
            .await?;

        Ok(Message::reducing_messages(messages))
    }

    #[deprecated(note = "Use the Room API instead.")]
    pub async fn load_messages_before(
        &self,
        from: &BareJid,
        before: impl Into<&MessageId> + Debug,
    ) -> Result<Page<Message>> {
        // TODO: See comment below
        // It might be possible that we have a holes in our cached messages, if we've synced with
        // the server only sporadically or in busy conversations. Our cache would still happily
        // return a page and report success since it found some messages. Do we always need a
        // server round trip?
        //
        // Local Remote
        //         1
        //   2     2
        //         3
        //         4
        //   5     5
        //   6     6

        let before: message::Id = before.into().as_ref().into();

        // If we have messages cached already return these without a round trip to the server…
        if let Some(cached_messages) = self
            .inner
            .data_cache
            .load_messages_before(from, Some(&before), MESSAGE_PAGE_SIZE)
            .await?
        {
            debug!("Returning cached messages for conversation {}…", from);
            return self
                .enriching_messages_from_cache(from, cached_messages)
                .await;
        }

        // We couldn't find any older messages but we need to have the one matching the id at least.
        // So we'll fetch that to translate the MessageId into a StanzaId for the server.
        let Some(stanza_id) = self.inner.data_cache.load_stanza_id(from, &before).await? else {
            return Err(format_err!(
                "Could not determine stanza_id for message with id {}",
                before
            ));
        };

        debug!("Loading messages for conversation {}…", from);
        let mam = self.client.get_mod::<MAM>();
        let (messages, fin) = mam
            .load_messages_in_chat(from, &stanza_id, None, Some(MESSAGE_PAGE_SIZE as usize))
            .await?;

        let Some(first_message) = messages.first() else {
            return Ok(Page {
                items: vec![],
                is_complete: true,
            });
        };

        let oldest_message_id: Option<message::Id> = if fin.complete == Complete::default() {
            first_message
                .forwarded
                .stanza
                .as_ref()
                .and_then(|m| m.id.clone())
        } else {
            None
        };

        let parsed_messages = messages
            .iter()
            .map(|msg| match MessageLike::try_from(msg) {
                Ok(mut msg) => {
                    msg.is_first_message = Some(msg.id.id()) == oldest_message_id.as_ref();
                    Ok(msg)
                }
                Err(err) => Err(err),
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.inner
            .data_cache
            .insert_messages(parsed_messages.iter())
            .await?;

        self.enriching_messages_from_cache(
            from,
            Page {
                items: parsed_messages,
                is_complete: fin.complete == Complete::default(),
            },
        )
        .await
    }

    #[deprecated(note = "Use the Room API instead.")]
    pub async fn load_messages_with_ids(
        &self,
        conversation: &BareJid,
        ids: &[MessageId],
    ) -> Result<Vec<Message>> {
        let ids = ids
            .iter()
            .map(|id| id.as_ref().into())
            .collect::<Vec<message::Id>>();
        let messages = self
            .inner
            .data_cache
            .load_messages_targeting(conversation, ids.as_slice(), None, true)
            .await?;
        debug!(
            "{}",
            messages
                .iter()
                .map(|m| format!("{:?}", m))
                .collect::<Vec<_>>()
                .join("\n")
        );
        Ok(Message::reducing_messages(messages))
    }

    #[deprecated(note = "Use the Room API instead.")]
    pub async fn send_message(
        &self,
        to: impl Into<Jid> + Debug,
        body: impl Into<String> + Debug,
    ) -> Result<()> {
        let chat = self.client.get_mod::<Chat>();
        chat.send_message(to, body, MessageType::Chat, Some(ChatState::Active))
    }

    #[deprecated(note = "Use the Room API instead.")]
    pub async fn update_message(
        &self,
        conversation: impl Into<Jid> + Debug,
        id: MessageId,
        body: impl Into<String> + Debug,
    ) -> Result<()> {
        let chat = self.client.get_mod::<Chat>();
        chat.update_message(id.into_inner().into(), conversation, body)
    }

    #[deprecated(note = "Use the Room API instead.")]
    pub async fn set_user_is_composing(
        &self,
        conversation: impl Into<Jid> + Debug,
        is_composing: bool,
    ) -> Result<()> {
        let chat = self.client.get_mod::<Chat>();
        chat.send_chat_state(
            conversation,
            if is_composing {
                ChatState::Composing
            } else {
                ChatState::Paused
            },
        )
    }

    #[deprecated(note = "Use the Room API instead.")]
    pub async fn load_composing_users(&self, conversation: &BareJid) -> Result<Vec<BareJid>> {
        // We currently do not support multi-user chats. So either our conversation partner is
        // typing or they are not.
        let conversation_partner_is_composing =
            self.inner.data_cache.load_chat_state(conversation).await?
                == Some(ChatState::Composing);

        if conversation_partner_is_composing {
            Ok(vec![conversation.clone()])
        } else {
            Ok(vec![])
        }
    }

    #[deprecated(note = "Use the Room API instead.")]
    pub async fn toggle_reaction_to_message(
        &self,
        conversation: impl Into<Jid> + Debug,
        id: MessageId,
        emoji: Emoji,
    ) -> Result<()> {
        let current_user = self.connected_jid()?.into_bare();
        let conversation = conversation.into().into_bare();
        let message_id = message::Id::from(id.into_inner());
        let message = self.load_message(&conversation, &message_id).await?;
        let mut emoji_found = false;

        let mut reactions = message
            .reactions
            .into_iter()
            .filter_map(|r| {
                if r.from.contains(&current_user) {
                    if r.emoji == emoji {
                        emoji_found = true;
                        return None;
                    }
                    Some(prose_xmpp::stanza::message::Emoji::from(
                        r.emoji.into_inner(),
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if !emoji_found {
            reactions.push(prose_xmpp::stanza::message::Emoji::from(emoji.into_inner()))
        }

        let chat = self.client.get_mod::<Chat>();
        chat.react_to_message(message_id, conversation, reactions)?;

        Ok(())
    }

    #[deprecated(note = "Use the Room API instead.")]
    pub async fn retract_message(
        &self,
        conversation: impl Into<Jid> + Debug,
        id: MessageId,
    ) -> Result<()> {
        let chat = self.client.get_mod::<Chat>();
        chat.retract_message(id.into_inner().into(), conversation)?;
        Ok(())
    }

    #[deprecated(note = "Use the Room API instead.")]
    pub async fn save_draft(&self, conversation: &BareJid, text: Option<&str>) -> Result<()> {
        self.inner.data_cache.save_draft(conversation, text).await?;
        Ok(())
    }

    #[deprecated(note = "Use the Room API instead.")]
    pub async fn load_draft(&self, conversation: &BareJid) -> Result<Option<String>> {
        Ok(self.inner.data_cache.load_draft(conversation).await?)
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    /// Takes a `Page` of `MessageLike` objects, fetches all `MessageLike` objects from the cache
    /// that modify messages in `page` and returns the reduced `Message`s.
    ///
    /// # Arguments
    ///
    /// * `conversation`: The conversation to which `page` belongs.
    /// * `page`: A page of messages.
    async fn enriching_messages_from_cache(
        &self,
        conversation: &BareJid,
        page: Page<MessageLike>,
    ) -> Result<Page<Message>> {
        let message_ids = page
            .items
            .iter()
            .map(|m| m.id.id().clone())
            .collect::<Vec<_>>();
        let last_message_id = page.items.last().unwrap().id.id();
        let modifiers = self
            .inner
            .data_cache
            .load_messages_targeting(&conversation, &message_ids, last_message_id, false)
            .await?;

        let reduced_messages =
            Message::reducing_messages(page.items.into_iter().chain(modifiers.into_iter()));

        Ok(Page {
            items: reduced_messages,
            is_complete: page.is_complete,
        })
    }

    #[deprecated(note = "Use the Room API instead.")]
    pub(super) async fn load_message(
        &self,
        conversation: &BareJid,
        message_id: &message::Id,
    ) -> Result<Message> {
        let ids = [MessageId::from(message_id.as_ref())];
        self.load_messages_with_ids(conversation, &ids)
            .await?
            .pop()
            .ok_or(format_err!("No message with id {}", ids[0]))
    }
}
