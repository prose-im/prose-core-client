// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::cmp::Ordering;
use std::usize;

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use gloo_utils::format::JsValueSerdeExt;
use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::IdbKeyRange;
use jid::BareJid;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use prose_xmpp::stanza::message;

use crate::data_cache::indexed_db::cache::{keys, IndexedDBDataCacheError};
use crate::data_cache::indexed_db::idb_database_ext::{IdbDatabaseExt, IdbObjectStoreExtGet};
use crate::data_cache::indexed_db::IndexedDBDataCache;
use crate::data_cache::MessageCache;
use crate::types::{MessageLike, Page};

use super::cache::Result;

#[derive(Serialize, Deserialize)]
struct IdbMessage {
    row_id: f64,
    message: MessageLike,
}

#[async_trait(? Send)]
impl MessageCache for IndexedDBDataCache {
    type Error = IndexedDBDataCacheError;

    async fn insert_messages<'a>(
        &self,
        messages: impl IntoIterator<Item = &'a MessageLike>,
    ) -> Result<()> {
        let tx = self
            .db
            .transaction_on_one_with_mode(keys::MESSAGES_STORE, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(keys::MESSAGES_STORE)?;
        let idx = store.index(keys::messages::ID_INDEX)?;

        for message in messages {
            if idx
                .get_key(&JsValue::from_str(message.id.as_ref()))?
                .await?
                .is_none()
            {
                store.put_val(&JsValue::from_serde(&message)?)?;
            }
        }
        tx.await.into_result()?;
        Ok(())
    }

    async fn load_messages_targeting<'a>(
        &self,
        conversation: &BareJid,
        targets: &[message::Id],
        newer_than: impl Into<Option<&'a message::Id>>,
        include_targeted_messages: bool,
    ) -> Result<Vec<MessageLike>> {
        let mut messages: Vec<MessageLike> = vec![];
        let distant_past: DateTime<FixedOffset> = Utc.timestamp_opt(0, 0).unwrap().into();

        let tx = self
            .db
            .transaction_on_one_with_mode(keys::MESSAGES_STORE, IdbTransactionMode::Readonly)?;

        let store = tx.object_store(keys::MESSAGES_STORE)?;
        let target_idx = store.index(keys::messages::TARGET_INDEX)?;
        let id_idx = store.index(keys::messages::ID_INDEX)?;

        let newer_than_id = newer_than.into();
        let newer_than = if let Some(newer_than) = newer_than_id {
            id_idx.timestamp_for_message(newer_than).await?
        } else {
            distant_past
        };

        async fn collect_messages<T: IdbObjectStoreExtGet>(
            store: &T,
            targets: &[message::Id],
            conversation: &BareJid,
            newer_than_id: &Option<&message::Id>,
            newer_than: DateTime<FixedOffset>,
            messages: &mut Vec<MessageLike>,
        ) -> Result<()> {
            for target in targets {
                let found_messages = store.get_all_values::<MessageLike>(target.as_ref()).await?;

                for message in found_messages {
                    if let Some(newer_than_id) = *newer_than_id {
                        if &message.id == newer_than_id {
                            continue;
                        }
                    }

                    if message.timestamp < newer_than
                        || (&message.from != conversation && &message.to != conversation)
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
    ) -> Result<Option<Page<MessageLike>>> {
        if max_count < 1 {
            return Ok(None);
        }

        let tx = self
            .db
            .transaction_on_one_with_mode(keys::MESSAGES_STORE, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(keys::MESSAGES_STORE)?;
        let timestamp_idx = store.index(keys::messages::TIMESTAMP_INDEX)?;
        let id_idx = store.index(keys::messages::ID_INDEX)?;

        let cursor = if let Some(older_than) = older_than {
            let older_than_timestamp = id_idx.timestamp_for_message(older_than).await?;
            let range = IdbKeyRange::lower_bound_with_open(
                &JsValue::from_serde(&older_than_timestamp)?,
                false,
            )
            .unwrap();
            timestamp_idx
                .open_cursor_with_range_and_direction(&range, IdbCursorDirection::Prev)?
                .await?
        } else {
            timestamp_idx
                .open_cursor_with_direction(IdbCursorDirection::Prev)?
                .await?
        };

        let Some(cursor) = cursor else {
            let has_items = store.count()?.await? > 0;

            return Ok(if has_items {
                Some(Page {
                    items: vec![],
                    is_complete: true,
                })
            } else {
                None
            });
        };

        let mut messages: Vec<IdbMessage> = vec![];

        loop {
            let message: MessageLike = cursor.value().into_serde()?;

            if Some(&message.id) != older_than
                && (&message.from == conversation || &message.to == conversation)
            {
                messages.push(IdbMessage {
                    row_id: cursor
                        .primary_key()
                        .and_then(|k| k.as_f64())
                        .ok_or(IndexedDBDataCacheError::InvalidMessageId)?,
                    message,
                });
            }

            if messages.len() >= max_count as usize || !cursor.continue_cursor()?.await? {
                break;
            }
        }

        messages.sort_by(
            |m1, m2| match m1.message.timestamp.cmp(&m2.message.timestamp) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => m1.row_id.total_cmp(&m2.row_id),
            },
        );

        Ok(Some(Page {
            items: messages.into_iter().map(|m| m.message).collect(),
            is_complete: false,
        }))
    }

    async fn load_messages_after(
        &self,
        conversation: &BareJid,
        newer_than: &message::Id,
        max_count: Option<u32>,
    ) -> Result<Vec<MessageLike>> {
        let max_count = max_count.map(|c| c as usize).unwrap_or(usize::MAX);
        if max_count < 1 {
            return Ok(vec![]);
        }

        let tx = self
            .db
            .transaction_on_one_with_mode(keys::MESSAGES_STORE, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(keys::MESSAGES_STORE)?;
        let timestamp_idx = store.index(keys::messages::TIMESTAMP_INDEX)?;
        let id_idx = store.index(keys::messages::ID_INDEX)?;

        let newer_than_timestamp = id_idx.timestamp_for_message(newer_than).await?;
        let range =
            IdbKeyRange::lower_bound_with_open(&JsValue::from_serde(&newer_than_timestamp)?, false)
                .unwrap();

        let cursor = timestamp_idx
            .open_cursor_with_range_and_direction(&range, IdbCursorDirection::Prev)?
            .await?;

        let Some(cursor) = cursor else {
            return Ok(vec![]);
        };

        let mut messages: Vec<IdbMessage> = vec![];

        loop {
            let message: MessageLike = cursor.value().into_serde()?;

            if &message.id != newer_than
                && (&message.from == conversation || &message.to == conversation)
            {
                messages.push(IdbMessage {
                    row_id: cursor
                        .primary_key()
                        .and_then(|k| k.as_f64())
                        .ok_or(IndexedDBDataCacheError::InvalidMessageId)?,
                    message,
                });
            }

            if messages.len() >= max_count || !cursor.continue_cursor()?.await? {
                break;
            }
        }

        messages.sort_by(
            |m1, m2| match m1.message.timestamp.cmp(&m2.message.timestamp) {
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
                Ordering::Equal => m1.row_id.total_cmp(&m2.row_id),
            },
        );

        Ok(messages.into_iter().map(|m| m.message).collect())
    }

    async fn load_stanza_id(
        &self,
        _conversation: &BareJid,
        message_id: &message::Id,
    ) -> Result<Option<message::stanza_id::Id>> {
        let message = self
            .db
            .get_value::<MessageLike>(keys::MESSAGES_STORE, message_id.to_string())
            .await?;
        let Some(message) = message else {
            return Ok(None);
        };

        Ok(message.stanza_id)
    }

    async fn save_draft(&self, conversation: &BareJid, text: Option<&str>) -> Result<()> {
        if let Some(text) = text {
            self.db
                .set_value(keys::DRAFTS_STORE, conversation.to_string(), &text)
                .await
        } else {
            self.db
                .delete_value(keys::DRAFTS_STORE, conversation.to_string())
                .await
        }
    }

    async fn load_draft(&self, conversation: &BareJid) -> Result<Option<String>> {
        self.db
            .get_value(keys::DRAFTS_STORE, conversation.to_string())
            .await
    }
}

#[async_trait(? Send)]
trait IdbObjectStoreMessageExt {
    async fn timestamp_for_message(
        &self,
        message_id: &message::Id,
    ) -> Result<DateTime<FixedOffset>>;
}

#[async_trait(? Send)]
impl IdbObjectStoreMessageExt for IdbIndex<'_> {
    async fn timestamp_for_message(
        &self,
        message_id: &message::Id,
    ) -> Result<DateTime<FixedOffset>> {
        let message = self
            .get_value::<MessageLike>(message_id.as_ref())
            .await?
            .ok_or(IndexedDBDataCacheError::InvalidMessageId)?;
        Ok(message.timestamp)
    }
}
