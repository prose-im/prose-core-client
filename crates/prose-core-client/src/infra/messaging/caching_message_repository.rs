// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::warn;

use prose_store::prelude::*;

use crate::domain::messaging::models::{MessageId, MessageLike, MessageTargetId, StanzaId};
use crate::domain::messaging::repos::MessagesRepository;
use crate::domain::shared::models::RoomId;
use crate::dtos::UserId;
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
        account: &UserId,
        room_id: &RoomId,
        id: &MessageId,
    ) -> Result<Vec<MessageLike>> {
        Ok(self.get_all(account, room_id, &[id.clone()]).await?)
    }

    async fn get_all(
        &self,
        account: &UserId,
        room_id: &RoomId,
        ids: &[MessageId],
    ) -> Result<Vec<MessageLike>> {
        warn!("Getting {} {} {:?}", account, room_id, ids);

        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;
        let collection = tx.readable_collection(MessageRecord::collection())?;

        let stanza_id_target_idx = collection.index(&MessageRecord::stanza_id_target_idx())?;
        let message_id_target_idx = collection.index(&MessageRecord::message_id_target_idx())?;
        let message_id_idx = collection.index(&MessageRecord::message_id_idx())?;

        let mut messages: Vec<MessageLike> = vec![];

        for id in ids {
            let message = message_id_idx
                .get::<_, MessageRecord>(&(account, room_id, id))
                .await?;

            messages.extend(
                &mut message_id_target_idx
                    .get_all_values::<MessageRecord>(
                        Query::Only((account.clone(), room_id.clone(), id.clone())),
                        Default::default(),
                        None,
                    )
                    .await?
                    .into_iter()
                    .map(MessageLike::from),
            );

            if let Some(stanza_id) = message.as_ref().and_then(|m| m.stanza_id.as_ref()) {
                messages.extend(
                    &mut stanza_id_target_idx
                        .get_all_values::<MessageRecord>(
                            Query::Only((account.clone(), room_id.clone(), stanza_id.clone())),
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
        account: &UserId,
        room_id: &RoomId,
        targeted_ids: &[MessageTargetId],
        newer_than: &DateTime<Utc>,
    ) -> Result<Vec<MessageLike>> {
        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;

        let collection = tx.readable_collection(MessageRecord::collection())?;
        let stanza_idx = collection.index(&MessageRecord::stanza_id_target_idx())?;
        let message_idx = collection.index(&MessageRecord::message_id_target_idx())?;

        let mut messages: Vec<MessageLike> = vec![];
        for id in targeted_ids {
            let targeting_messages = match id {
                MessageTargetId::MessageId(id) => {
                    message_idx
                        .get_all_values::<MessageRecord>(
                            Query::Only((account.clone(), room_id.clone(), id.clone())),
                            Default::default(),
                            None,
                        )
                        .await?
                }
                MessageTargetId::StanzaId(id) => {
                    stanza_idx
                        .get_all_values::<MessageRecord>(
                            Query::Only((account.clone(), room_id.clone(), id.clone())),
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

    async fn contains(&self, account: &UserId, room_id: &RoomId, id: &MessageId) -> Result<bool> {
        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;
        let collection = tx.readable_collection(MessageRecord::collection())?;
        let idx = collection.index(&MessageRecord::message_id_idx())?;
        let flag = idx
            .contains_key(&(account.clone(), room_id.clone(), id.clone()))
            .await?;
        Ok(flag)
    }

    async fn append(
        &self,
        account: &UserId,
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

    async fn clear_cache(&self, account: &UserId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[MessageRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(MessageRecord::collection())?;
        collection
            .delete_all_in_index(&MessageRecord::account_idx(), Query::Only(account.clone()))
            .await?;
        tx.commit().await?;

        Ok(())
    }

    async fn resolve_message_id(
        &self,
        account: &UserId,
        room_id: &RoomId,
        stanza_id: &StanzaId,
    ) -> Result<Option<MessageId>> {
        let tx = self
            .store
            .transaction_for_reading(&[MessageRecord::collection()])
            .await?;
        let collection = tx.readable_collection(MessageRecord::collection())?;
        let stanza_idx = collection.index(&MessageRecord::stanza_id_idx())?;
        let message = stanza_idx
            .get::<_, MessageRecord>(&(account.clone(), room_id.clone(), stanza_id.clone()))
            .await?;
        Ok(message.and_then(|m| m.message_id.into_original_id()))
    }
}
