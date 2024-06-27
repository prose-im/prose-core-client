// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use prose_store::prelude::*;
use prose_store::{define_entity, RawKey};
use prose_xmpp::mods::AvatarData;

use crate::domain::shared::models::{AccountId, AvatarId};
use crate::domain::user_info::models::{AvatarInfo, PlatformImage};
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

#[derive(Serialize, Deserialize)]
pub struct AvatarRecord {
    id: String,
    account: AccountId,
    user_id: UserId,
    avatar_id: AvatarId,
    mime_type: String,
    base64_data: String,
}

impl AvatarRecord {
    fn new(
        account: &AccountId,
        user_id: &UserId,
        image: &AvatarData,
        metadata: &AvatarInfo,
    ) -> Self {
        Self {
            id: format!("{}.{}", account, user_id),
            account: account.clone(),
            user_id: user_id.clone(),
            avatar_id: metadata.checksum.clone(),
            mime_type: metadata.mime_type.clone(),
            base64_data: image.base64().to_string(),
        }
    }
}

mod columns {
    pub const ACCOUNT: &str = "account";
    pub const USER_ID: &str = "user_id";
    pub const AVATAR_ID: &str = "avatar_id";
}

define_entity!(AvatarRecord, "avatar",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    // We're only saving one avatar per user
    user_idx => { columns: [columns::ACCOUNT, columns::USER_ID], unique: true },
    avatar_idx => { columns: [columns::ACCOUNT, columns::USER_ID, columns::AVATAR_ID], unique: true }
);

impl KeyType for AvatarId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl AvatarCache for StoreAvatarCache {
    async fn cache_avatar_image(
        &self,
        account: &AccountId,
        user_id: &UserId,
        image: &AvatarData,
        metadata: &AvatarInfo,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[AvatarRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(AvatarRecord::collection())?;
        collection.put_entity(&AvatarRecord::new(account, user_id, image, metadata))?;
        tx.commit().await?;
        Ok(())
    }

    async fn has_cached_avatar_image(
        &self,
        account: &AccountId,
        user_id: &UserId,
        image_checksum: &AvatarId,
    ) -> Result<bool> {
        let tx = self
            .store
            .transaction_for_reading(&[AvatarRecord::collection()])
            .await?;
        let collection = tx.readable_collection(AvatarRecord::collection())?;
        let idx = collection.index(&AvatarRecord::avatar_idx())?;
        let contains_image = idx
            .contains_key(&(account, user_id, image_checksum))
            .await?;
        Ok(contains_image)
    }

    async fn cached_avatar_image(
        &self,
        account: &AccountId,
        user_id: &UserId,
        image_checksum: &AvatarId,
    ) -> Result<Option<PlatformImage>> {
        let tx = self
            .store
            .transaction_for_reading(&[AvatarRecord::collection()])
            .await?;
        let collection = tx.readable_collection(AvatarRecord::collection())?;
        let idx = collection.index(&AvatarRecord::avatar_idx())?;

        return Ok(idx
            .get::<_, AvatarRecord>(&(account, user_id, image_checksum))
            .await?
            .map(|record| format!("data:{};base64,{}", record.mime_type, record.base64_data)));
    }

    async fn delete_all_cached_images(&self, account: &AccountId) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[AvatarRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(AvatarRecord::collection())?;
        collection
            .delete_all_in_index(&AvatarRecord::account_idx(), Query::Only(account))
            .await?;
        tx.commit().await?;
        Ok(())
    }
}
