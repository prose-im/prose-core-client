// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_store::prelude::*;
use prose_store::RawKey;
use prose_xmpp::mods::AvatarData;

use crate::domain::user_info::models::{AvatarImageId, AvatarInfo, PlatformImage};
use crate::dtos::UserId;
use crate::infra::avatars::AvatarCache;

pub struct StoreAvatarCache {
    store: Store<PlatformDriver>,
}

impl StoreAvatarCache {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

#[entity]
pub struct AvatarRecord {
    id: AvatarImageId,
    mime_type: String,
    base64_data: String,
}

impl KeyType for AvatarImageId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl AvatarCache for StoreAvatarCache {
    async fn cache_avatar_image(
        &self,
        _jid: &UserId,
        image: &AvatarData,
        metadata: &AvatarInfo,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[AvatarRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(AvatarRecord::collection())?;
        collection.put_entity(&AvatarRecord {
            id: metadata.checksum.clone(),
            mime_type: metadata.mime_type.clone(),
            base64_data: image.base64().to_string(),
        })?;
        tx.commit().await?;
        Ok(())
    }

    async fn has_cached_avatar_image(
        &self,
        _jid: &UserId,
        image_checksum: &AvatarImageId,
    ) -> Result<bool> {
        let tx = self
            .store
            .transaction_for_reading(&[AvatarRecord::collection()])
            .await?;
        let collection = tx.readable_collection(AvatarRecord::collection())?;
        let contains_image = collection.contains_key(image_checksum).await?;
        Ok(contains_image)
    }

    async fn cached_avatar_image(
        &self,
        _jid: &UserId,
        image_checksum: &AvatarImageId,
    ) -> Result<Option<PlatformImage>> {
        let tx = self
            .store
            .transaction_for_reading(&[AvatarRecord::collection()])
            .await?;
        let collection = tx.readable_collection(AvatarRecord::collection())?;
        let Some(record) = collection.get::<_, AvatarRecord>(image_checksum).await? else {
            return Ok(None);
        };
        Ok(Some(format!(
            "data:{};base64,{}",
            record.mime_type, record.base64_data
        )))
    }

    async fn delete_all_cached_images(&self) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[AvatarRecord::collection()])
            .await?;
        tx.truncate_collections(&[AvatarRecord::collection()])?;
        tx.commit().await?;
        Ok(())
    }
}
