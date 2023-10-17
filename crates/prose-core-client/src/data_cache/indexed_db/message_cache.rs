// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::usize;

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use jid::BareJid;
use prose_store::prelude::*;
use prose_wasm_utils::SendUnlessWasm;
use serde::{Deserialize, Serialize};

use prose_xmpp::stanza::message;

use crate::data_cache::indexed_db::cache::{keys, CacheError};
use crate::data_cache::indexed_db::IndexedDBDataCache;
use crate::data_cache::MessageCache;
use crate::types::{MessageLike, Page};

#[derive(Serialize, Deserialize)]
struct IdbMessage {
    row_id: f64,
    message: MessageLike,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl<D: Driver> MessageCache for IndexedDBDataCache<D> {
    type Error = CacheError;

    async fn insert_messages<'a>(
        &self,
        messages: impl IntoIterator<Item = &'a MessageLike> + SendUnlessWasm,
    ) -> Result<(), Self::Error> {
        let tx = self
            .db
            .transaction_for_reading_and_writing(&[keys::MESSAGES_STORE])
            .await?;

        {
            let collection = tx.writeable_collection(keys::MESSAGES_STORE)?;
            for message in messages {
                collection.put(message.id.id().as_ref(), &message)?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn load_messages_targeting<'a>(
        &self,
        conversation: &BareJid,
        targets: &[message::Id],
        newer_than: impl Into<Option<&'a message::Id>> + SendUnlessWasm,
        include_targeted_messages: bool,
    ) -> Result<Vec<MessageLike>, Self::Error> {
        let mut messages: Vec<MessageLike> = vec![];
        let distant_past: DateTime<FixedOffset> = Utc.timestamp_opt(0, 0).unwrap().into();

        let tx = self
            .db
            .transaction_for_reading(&[keys::MESSAGES_STORE])
            .await?;

        let store = tx.readable_collection(keys::MESSAGES_STORE)?;
        let target_idx = store.index(keys::messages::TARGET_INDEX)?;
        let id_idx = store.index(keys::messages::ID_INDEX)?;

        let newer_than_id = newer_than.into();
        let newer_than = if let Some(newer_than) = newer_than_id {
            id_idx
                .get::<_, MessageLike>(newer_than.as_ref())
                .await?
                .ok_or(CacheError::InvalidMessageId)?
                .timestamp
        } else {
            distant_past
        };

        async fn collect_messages<'tx, E: StoreError>(
            store: &impl ReadableCollection<'tx, Error = E>,
            targets: &[message::Id],
            conversation: &BareJid,
            newer_than_id: &Option<&message::Id>,
            newer_than: DateTime<FixedOffset>,
            messages: &mut Vec<MessageLike>,
        ) -> Result<(), CacheError> {
            for target in targets {
                let found_messages = store
                    .get_all_values::<MessageLike>(
                        Query::Only(target.as_ref()),
                        QueryDirection::Backward,
                        None,
                    )
                    .await?;

                for message in found_messages {
                    if let Some(newer_than_id) = *newer_than_id {
                        if message.id.id() == newer_than_id {
                            continue;
                        }
                    }

                    if message.timestamp < newer_than
                        || !message.belongs_to_conversation(conversation)
                    {
                        continue;
                    }

                    messages.push(message);
                }
            }

            Ok(())
        }

        collect_messages(
            &target_idx,
            targets,
            conversation,
            &newer_than_id,
            newer_than,
            &mut messages,
        )
        .await?;

        if include_targeted_messages {
            collect_messages(
                &id_idx,
                targets,
                conversation,
                &newer_than_id,
                newer_than,
                &mut messages,
            )
            .await?;
        }

        messages.sort_by_key(|m| m.timestamp);
        Ok(messages)
    }

    async fn load_messages_before(
        &self,
        conversation: &BareJid,
        older_than: Option<&message::Id>,
        max_count: u32,
    ) -> Result<Option<Page<MessageLike>>, Self::Error> {
        if max_count < 1 {
            return Ok(None);
        }

        let tx = self
            .db
            .transaction_for_reading(&[keys::MESSAGES_STORE])
            .await?;
        let store = tx.readable_collection(keys::MESSAGES_STORE)?;
        let timestamp_idx = store.index(keys::messages::TIMESTAMP_INDEX)?;
        let id_idx = store.index(keys::messages::ID_INDEX)?;

        let query = if let Some(older_than) = older_than {
            let older_than_timestamp = id_idx
                .get::<_, MessageLike>(older_than.as_ref())
                .await?
                .ok_or(CacheError::InvalidMessageId)?
                .timestamp;

            Query::from_range(older_than_timestamp..)
        } else {
            Query::from_range(..)
        };

        let mut messages = timestamp_idx
            .get_all_filtered::<MessageLike, _>(
                query,
                QueryDirection::Backward,
                Some(max_count as usize),
                |_, message| {
                    if Some(message.id.id()) == older_than {
                        return None;
                    }
                    message
                        .belongs_to_conversation(conversation)
                        .then_some(message)
                },
            )
            .await?;
        messages.reverse();

        Ok(Some(Page {
            items: messages,
            is_complete: false,
        }))
    }

    async fn load_messages_after(
        &self,
        conversation: &BareJid,
        newer_than: &message::Id,
        max_count: Option<u32>,
    ) -> Result<Vec<MessageLike>, Self::Error> {
        let max_count = max_count.map(|c| c as usize).unwrap_or(usize::MAX);
        if max_count < 1 {
            return Ok(vec![]);
        }

        let tx = self
            .db
            .transaction_for_reading(&[keys::MESSAGES_STORE])
            .await?;
        let store = tx.readable_collection(keys::MESSAGES_STORE)?;
        let timestamp_idx = store.index(keys::messages::TIMESTAMP_INDEX)?;
        let id_idx = store.index(keys::messages::ID_INDEX)?;

        let newer_than_timestamp = id_idx
            .get::<_, MessageLike>(newer_than.as_ref())
            .await?
            .ok_or(CacheError::InvalidMessageId)?
            .timestamp;
        let query = Query::from_range(newer_than_timestamp..);

        let mut messages = timestamp_idx
            .get_all_filtered::<MessageLike, _>(
                query,
                QueryDirection::Backward,
                Some(max_count),
                |_, message| {
                    if message.id.id() == newer_than {
                        return None;
                    }
                    message
                        .belongs_to_conversation(conversation)
                        .then_some(message)
                },
            )
            .await?;
        messages.reverse();

        Ok(messages)
    }

    async fn load_stanza_id(
        &self,
        _conversation: &BareJid,
        message_id: &message::Id,
    ) -> Result<Option<message::stanza_id::Id>, Self::Error> {
        let message = self
            .db
            .get::<_, MessageLike>(keys::MESSAGES_STORE, message_id.as_ref())
            .await?;
        let Some(message) = message else {
            return Ok(None);
        };

        Ok(message.stanza_id)
    }

    async fn save_draft(
        &self,
        conversation: &BareJid,
        text: Option<&str>,
    ) -> Result<(), Self::Error> {
        if let Some(text) = text {
            self.db
                .put(keys::DRAFTS_STORE, &conversation.to_string(), &text)
                .await?;
        } else {
            self.db
                .delete(keys::DRAFTS_STORE, &conversation.to_string())
                .await?;
        }
        Ok(())
    }

    async fn load_draft(&self, conversation: &BareJid) -> Result<Option<String>, Self::Error> {
        let draft = self
            .db
            .get(keys::DRAFTS_STORE, &conversation.to_string())
            .await?;
        Ok(draft)
    }
}

trait MessageLikeExt {
    fn belongs_to_conversation(&self, conversation: &BareJid) -> bool;
}

impl MessageLikeExt for MessageLike {
    fn belongs_to_conversation(&self, conversation: &BareJid) -> bool {
        &self.from == conversation || self.to.as_ref() == Some(conversation)
    }
}
