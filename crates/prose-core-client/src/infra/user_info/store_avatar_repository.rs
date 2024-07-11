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

use crate::domain::shared::models::{AccountId, AvatarId, ParticipantId, ParticipantIdRef};
use crate::domain::user_info::models::{Avatar, AvatarInfo, AvatarSource, PlatformImage};
use crate::domain::user_info::repos::AvatarRepository;

pub struct StoreAvatarRepository {
    store: Store<PlatformDriver>,
}

impl StoreAvatarRepository {
    pub fn new(store: Store<PlatformDriver>) -> Self {
        Self { store }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AvatarRecord {
    id: String,
    account: AccountId,
    participant_id: ParticipantId,
    avatar_id: AvatarId,
    mime_type: String,
    base64_data: String,
}

impl AvatarRecord {
    fn new(
        account: &AccountId,
        participant_id: ParticipantId,
        image: &AvatarData,
        metadata: &AvatarInfo,
    ) -> Self {
        Self {
            id: format!("{account}.{}", participant_id.to_ref().to_raw_key_string()),
            account: account.clone(),
            participant_id,
            avatar_id: metadata.checksum.clone(),
            mime_type: metadata.mime_type.clone(),
            base64_data: image.base64().to_string(),
        }
    }
}

mod columns {
    pub const ACCOUNT: &str = "account";
    pub const PARTICIPANT_ID: &str = "participant_id";
    pub const AVATAR_ID: &str = "avatar_id";
}

define_entity!(AvatarRecord, "avatar",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    // We're only saving one avatar per user
    user_idx => { columns: [columns::ACCOUNT, columns::PARTICIPANT_ID], unique: true },
    avatar_idx => { columns: [columns::ACCOUNT, columns::PARTICIPANT_ID, columns::AVATAR_ID], unique: true }
);

impl KeyType for AvatarId {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl AvatarRepository for StoreAvatarRepository {
    async fn get(&self, account: &AccountId, avatar: &Avatar) -> Result<Option<PlatformImage>> {
        let tx = self
            .store
            .transaction_for_reading(&[AvatarRecord::collection()])
            .await?;
        let collection = tx.readable_collection(AvatarRecord::collection())?;
        let idx = collection.index(&AvatarRecord::avatar_idx())?;
        let participant_id = match &avatar.source {
            AvatarSource::Pep { owner, .. } => ParticipantIdRef::User(owner),
            AvatarSource::Vcard { owner } => owner.to_ref(),
        };

        return Ok(idx
            .get::<_, AvatarRecord>(&(account, participant_id, &avatar.id))
            .await?
            .map(|record| format!("data:{};base64,{}", record.mime_type, record.base64_data)));
    }

    async fn set(
        &self,
        account: &AccountId,
        participant_id: ParticipantIdRef<'_>,
        metadata: &AvatarInfo,
        image: &AvatarData,
    ) -> Result<()> {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[AvatarRecord::collection()])
            .await?;
        let collection = tx.writeable_collection(AvatarRecord::collection())?;
        collection.put_entity(&AvatarRecord::new(
            account,
            participant_id.to_owned(),
            image,
            metadata,
        ))?;
        tx.commit().await?;
        Ok(())
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
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
