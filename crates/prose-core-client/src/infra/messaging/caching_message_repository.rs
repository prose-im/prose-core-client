// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::Bound;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use prose_store::prelude::*;

use crate::domain::messaging::models::{
    ArchivedMessageRef, MessageId, MessageIdTriple, MessageLike, MessageRemoteId, MessageServerId,
    MessageTargetId,
};
use crate::domain::messaging::repos::MessagesRepository;
use crate::domain::shared::models::{AccountId, RoomId};
use crate::infra::messaging::MessageRecord;

// TODO: Incorporate MessageArchiveService, cache complete pages loaded from the server

pub struct CachingMessageRepository {
    store: Store<PlatformDriver>,
}

impl CachingMessageRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl MessagesRepository for CachingMessageRepository {
    async fn get(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        id: &MessageId,
    ) -> Result<Vec<MessageLike>> {
        Ok(self.get_all(account, room_id, &[id.clone()]).await?)
    }

    async fn get_all(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        ids: &[MessageId],
    ) -> Result<Vec<MessageLike>> {
        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;
        let collection = tx.readable_collection(MessageRecord::collection())?;

        let stanza_id_target_idx = collection.index(&MessageRecord::server_id_target_idx())?;
        let message_id_target_idx = collection.index(&MessageRecord::remote_id_target_idx())?;
        let message_id_idx = collection.index(&MessageRecord::message_id_idx())?;

        let mut messages: Vec<MessageLike> = vec![];

        for id in ids {
            let message = message_id_idx
                .get::<_, MessageRecord>(&(account, room_id, id))
                .await?;

            if let Some(remote_id) = message.as_ref().and_then(|m| m.remote_id.as_ref()) {
                messages.extend(
                    &mut message_id_target_idx
                        .get_all_values::<MessageRecord>(
                            Query::Only((account, room_id, remote_id)),
                            Default::default(),
                            None,
                        )
                        .await?
                        .into_iter()
                        .map(MessageLike::from),
                );
            }

            if let Some(stanza_id) = message.as_ref().and_then(|m| m.server_id.as_ref()) {
                messages.extend(
                    &mut stanza_id_target_idx
                        .get_all_values::<MessageRecord>(
                            Query::Only((account, room_id, stanza_id)),
                            Default::default(),
                            None,
                        )
                        .await?
                        .into_iter()
                        .map(MessageLike::from),
                );
            }

            if let Some(message) = message {
                messages.push(message.into());
            }
        }

        messages.sort_by_key(|msg| msg.timestamp);
        Ok(messages)
    }

    async fn get_messages_targeting(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        targeted_ids: &[MessageTargetId],
        newer_than: &DateTime<Utc>,
    ) -> Result<Vec<MessageLike>> {
        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;

        let collection = tx.readable_collection(MessageRecord::collection())?;
        let stanza_idx = collection.index(&MessageRecord::server_id_target_idx())?;
        let message_idx = collection.index(&MessageRecord::remote_id_target_idx())?;

        let mut messages: Vec<MessageLike> = vec![];
        for id in targeted_ids {
            let targeting_messages = match id {
                MessageTargetId::RemoteId(id) => {
                    message_idx
                        .get_all_values::<MessageRecord>(
                            Query::Only((account, room_id, id)),
                            Default::default(),
                            None,
                        )
                        .await?
                }
                MessageTargetId::ServerId(id) => {
                    stanza_idx
                        .get_all_values::<MessageRecord>(
                            Query::Only((account, room_id, id)),
                            Default::default(),
                            None,
                        )
                        .await?
                }
            };

            messages.extend(
                &mut targeting_messages
                    .into_iter()
                    .filter(|msg| &msg.timestamp > newer_than)
                    .map(MessageLike::from),
            );
        }

        messages.sort_by_key(|msg| msg.timestamp);
        Ok(messages)
    }

    async fn contains(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        id: &MessageServerId,
    ) -> Result<bool> {
        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;
        let collection = tx.readable_collection(MessageRecord::collection())?;
        let idx = collection.index(&MessageRecord::server_id_idx())?;
        let flag = idx.contains_key(&(account, room_id, id)).await?;
        Ok(flag)
    }

    async fn append(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        messages: &[MessageLike],
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[MessageRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(MessageRecord::collection())?;
        for message in messages {
            collection.put_entity(&MessageRecord::from_message(
                account.clone(),
                room_id.clone(),
                message.clone(),
            ))?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[MessageRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(MessageRecord::collection())?;
        collection
            .delete_all_in_index(&MessageRecord::account_idx(), Query::Only(account))
            .await?;
        tx.commit().await?;

        Ok(())
    }

    async fn resolve_server_id(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        server_id: &MessageServerId,
    ) -> Result<Option<MessageIdTriple>> {
        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;
        let collection = tx.readable_collection(MessageRecord::collection())?;
        let stanza_idx = collection.index(&MessageRecord::server_id_idx())?;
        let message = stanza_idx
            .get::<_, MessageRecord>(&(account, room_id, server_id))
            .await?;
        Ok(message.map(|m| MessageIdTriple {
            id: m.message_id,
            remote_id: m.remote_id,
            server_id: m.server_id,
        }))
    }

    async fn resolve_remote_id(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        remote_id: &MessageRemoteId,
    ) -> Result<Option<MessageIdTriple>> {
        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;
        let collection = tx.readable_collection(MessageRecord::collection())?;
        let remote_id_idx = collection.index(&MessageRecord::remote_id_idx())?;
        let message = remote_id_idx
            .get::<_, MessageRecord>(&(account, room_id, remote_id))
            .await?;
        Ok(message.map(|m| MessageIdTriple {
            id: m.message_id,
            remote_id: m.remote_id,
            server_id: m.server_id,
        }))
    }

    async fn resolve_message_id(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        id: &MessageId,
    ) -> Result<Option<MessageIdTriple>> {
        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;
        let collection = tx.readable_collection(MessageRecord::collection())?;
        let message_id_idx = collection.index(&MessageRecord::message_id_idx())?;
        let message = message_id_idx
            .get::<_, MessageRecord>(&(account, room_id, id))
            .await?;
        Ok(message.map(|m| MessageIdTriple {
            id: m.message_id,
            remote_id: m.remote_id,
            server_id: m.server_id,
        }))
    }

    async fn get_last_received_message(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        before: Option<DateTime<Utc>>,
    ) -> Result<Option<ArchivedMessageRef>> {
        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;
        let collection = tx.readable_collection(MessageRecord::collection())?;
        let room_idx = collection.index(&MessageRecord::room_idx())?;
        let before = before.unwrap_or(DateTime::<Utc>::MAX_UTC);
        let (message_ref, is_placeholder) = room_idx
            .fold::<MessageRecord, (ArchivedMessageRef, bool)>(
                Query::Only((account, room_id)),
                (
                    ArchivedMessageRef {
                        stanza_id: "".into(),
                        timestamp: DateTime::<Utc>::MIN_UTC,
                    },
                    true,
                ),
                |(result, is_placeholder), (_, message)| {
                    if message.timestamp >= before || message.timestamp < result.timestamp {
                        return (result, is_placeholder);
                    }

                    let Some(stanza_id) = message.server_id else {
                        return (result, is_placeholder);
                    };

                    (
                        ArchivedMessageRef {
                            stanza_id,
                            timestamp: message.timestamp,
                        },
                        false,
                    )
                },
            )
            .await?;

        if is_placeholder {
            return Ok(None);
        }

        Ok(Some(message_ref))
    }

    async fn get_messages_after(
        &self,
        account: &AccountId,
        room_id: &RoomId,
        after: DateTime<Utc>,
    ) -> Result<Vec<MessageLike>> {
        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;
        let collection = tx.readable_collection(MessageRecord::collection())?;
        let room_idx = collection.index(&MessageRecord::timestamp_idx())?;

        let messages = room_idx
            .get_all_filtered::<MessageRecord, MessageLike>(
                Query::Range {
                    start: Bound::Included((account, room_id, &after)),
                    end: Bound::Included((account, room_id, &DateTime::<Utc>::MAX_UTC)),
                },
                QueryDirection::default(),
                None,
                |_, message| (message.timestamp > after).then_some(MessageLike::from(message)),
            )
            .await?;

        Ok(messages)
    }
}
