use std::fmt::Debug;

use anyhow::{format_err, Result};
use jid::{BareJid, Jid};
use microtype::Microtype;
use tracing::{debug, info, instrument};
use xmpp_parsers::mam::Complete;

use prose_domain::{Emoji, Message, MessageId};
use prose_xmpp::mods::{Chat, MAM};
use prose_xmpp::stanza::message;
use prose_xmpp::stanza::message::ChatState;

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::domain_ext::MessageExt;
use crate::types::{MessageLike, Page};

use super::Client;

const MESSAGE_PAGE_SIZE: u32 = 50;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    pub async fn load_latest_messages(
        &self,
        from: &BareJid,
        since: impl Into<Option<&MessageId>> + Debug,
        load_from_server: bool,
    ) -> Result<Vec<Message>> {
        // TODO: See comment below
        // It's possible that newly loaded messages affect already visible ones in the client. In
        // this case we'll need to generate the appropriate `ClientEvent`s.

        // TODO: See comment below
        // It might also be possible that we do not receive the absolute last message from the
        // server if more than MESSAGE_PAGE_SIZE messages were sent since the last message we've
        // seen. In that case we need to compare the fin element's last id with the stanza id of
        // the last message to see if we've received it. Otherwise we'll need

        let since: Option<message::Id> = since.into().map(|id| id.as_ref().into());

        let mut messages = if let Some(since) = &since {
            info!(
                "Loading messages in conversation {} after {} from local cache…",
                from, since
            );
            self.inner
                .data_cache
                .load_messages_after(from, since, Some(MESSAGE_PAGE_SIZE))
                .await?
        } else {
            info!(
                "Loading last page of messages in conversation {} from local cache…",
                from
            );
            self.inner
                .data_cache
                .load_messages_before(from, None, MESSAGE_PAGE_SIZE)
                .await?
                .map(|page| page.items)
                .unwrap_or_else(|| vec![])
        };

        info!("Found {} messages in local cache.", messages.len());

        // We take either the stanza_id of the last cached message or the first stanza_id that is
        // followed by a local message for which we don't know the stanza_id yet. This way we're
        // syncing up with the server.
        let stanza_id = 'outer: loop {
            for (l, r) in messages.iter().zip(messages.iter().skip(1)) {
                if let (Some(l), None) = (&l.stanza_id, &r.stanza_id) {
                    break 'outer Some(l);
                }
            }
            break messages.last().and_then(|m| m.stanza_id.as_ref());
        };

        if load_from_server {
            info!("Loading messages from server since {:?}…", stanza_id);

            let mam = self.client.get_mod::<MAM>();
            let result = mam
                .load_messages_in_chat(from, None, stanza_id, Some(MESSAGE_PAGE_SIZE as usize))
                .await?;

            let mut remote_messages = result
                .0
                .iter()
                .map(|msg| MessageLike::try_from(msg))
                .collect::<Result<Vec<_>, _>>()?;

            info!("Found {} messages. Saving to cache…", remote_messages.len());
            self.inner
                .data_cache
                .insert_messages(remote_messages.iter())
                .await?;

            // Remove all messages from the tail of the local messages including the message that
            // matches the first message returned from the server so that we don't have any
            // duplicates but the latest remote data in our vec.
            //
            // Local Remote
            //   1
            //   2
            //   3     3
            //   4     4
            //   5     5

            if let Some(first_remote_message_id) = remote_messages.first().map(|m| &m.id) {
                let cutoff_idx = messages.iter().rev().enumerate().find_map(|(idx, msg)| {
                    if &msg.id == first_remote_message_id {
                        Some(messages.len() - idx - 1)
                    } else {
                        None
                    }
                });

                if let Some(cutoff_idx) = cutoff_idx {
                    debug!(
                        "Truncating local messages to messages before {:?} at index {:?}",
                        first_remote_message_id, cutoff_idx
                    );
                    messages.truncate(cutoff_idx);
                }
            }

            messages.append(&mut remote_messages);
        } else {
            info!("Skipping server round trip.")
        }

        Ok(Message::reducing_messages(messages))
    }

    #[instrument]
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
            info!("Returning cached messages for conversation {}…", from);
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

        info!("Loading messages for conversation {}…", from);
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
                    msg.is_first_message = Some(&msg.id) == oldest_message_id.as_ref();
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

    #[instrument]
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

    #[instrument]
    pub async fn send_message(
        &self,
        to: impl Into<Jid> + Debug,
        body: impl Into<String> + Debug,
    ) -> Result<()> {
        let chat = self.client.get_mod::<Chat>();
        chat.send_message(to, body, Some(ChatState::Active))
    }

    #[instrument]
    pub async fn update_message(
        &self,
        conversation: impl Into<Jid> + Debug,
        id: MessageId,
        body: impl Into<String> + Debug,
    ) -> Result<()> {
        let chat = self.client.get_mod::<Chat>();
        chat.update_message(id.into_inner().into(), conversation, body)
    }

    #[instrument]
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

    #[instrument]
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

    #[instrument]
    pub async fn toggle_reaction_to_message(
        &self,
        conversation: impl Into<Jid> + Debug,
        id: MessageId,
        emoji: Emoji,
    ) -> Result<()> {
        let current_user = BareJid::from(self.connected_jid()?);
        let conversation = BareJid::from(conversation.into());
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

    #[instrument]
    pub async fn retract_message(
        &self,
        conversation: impl Into<Jid> + Debug,
        id: MessageId,
    ) -> Result<()> {
        let chat = self.client.get_mod::<Chat>();
        chat.retract_message(id.into_inner().into(), conversation)?;
        Ok(())
    }

    pub async fn save_draft(&self, conversation: &BareJid, text: Option<&str>) -> Result<()> {
        self.inner.data_cache.save_draft(conversation, text).await?;
        Ok(())
    }

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
        let message_ids = page.items.iter().map(|m| m.id.clone()).collect::<Vec<_>>();
        let last_message_id = &page.items.last().unwrap().id;
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

    async fn load_message(
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
