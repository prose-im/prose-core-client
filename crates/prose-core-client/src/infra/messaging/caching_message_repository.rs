// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;

use prose_store::prelude::*;
use prose_store::RawKey;

use crate::domain::messaging::models::{Message, MessageId, MessageLike, MessageLikeId};
use crate::domain::messaging::repos::MessagesRepository;

// TODO: Incorporate MessageArchiveService, cache complete pages loaded from the server

pub struct CachingMessageRepository {
    store: Store<PlatformDriver>,
}

impl CachingMessageRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

pub type MessagesRecord = MessageLike;

impl Entity for MessageLike {
    type ID = MessageLikeId;

    fn id(&self) -> &Self::ID {
        &self.id
    }

    fn collection() -> &'static str {
        "messages"
    }

    fn indexes() -> Vec<IndexSpec> {
        vec![IndexSpec::builder("target").build()]
    }
}

impl KeyType for MessageLikeId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

impl KeyType for MessageId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl MessagesRepository for CachingMessageRepository {
    async fn get(&self, room_id: &BareJid, id: &MessageId) -> Result<Option<Message>> {
        Ok(self.get_all(room_id, &[id]).await?.pop())
    }

    async fn get_all(&self, _room_id: &BareJid, ids: &[&MessageId]) -> Result<Vec<Message>> {
        let tx = self
            .store
            .transaction_for_reading(&[MessagesRecord::collection()])
            .await?;
        let collection = tx.readable_collection(MessagesRecord::collection())?;
        let idx = collection.index("target")?;

        let mut messages: Vec<MessageLike> = vec![];
        for id in ids {
            if let Some(message) = collection.get(*id).await? {
                messages.push(message);
            }
            messages.append(
                &mut idx
                    .get_all_values(Query::Only((*id).clone()), Default::default(), None)
                    .await?,
            );
        }

        messages.sort_by_key(|msg| msg.timestamp);

        Ok(Message::reducing_messages(messages))
    }

    async fn append(&self, _room_id: &BareJid, messages: &[&MessageLike]) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[MessagesRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(MessagesRecord::collection())?;
        for message in messages {
            collection.put_entity(*message)?;
        }
        tx.commit().await?;
        Ok(())
    }

    async fn clear_cache(&self) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[MessagesRecord::collection()])
            .await?;
        tx.truncate_collections(&[MessagesRecord::collection()])?;
        tx.commit().await?;
        Ok(())
    }
}
