// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use indexed_db_futures::prelude::*;
use jid::BareJid;
use prose_xmpp::mods::AvatarData;
use wasm_bindgen::JsValue;

use crate::avatar_cache::AvatarCache;
use prose_xmpp::stanza::avatar::ImageId;

use crate::data_cache::indexed_db::cache::{keys, IndexedDBDataCacheError};
use crate::data_cache::indexed_db::idb_database_ext::{IdbDatabaseExt, IdbObjectStoreExtGet};
use crate::data_cache::indexed_db::IndexedDBDataCache;
use crate::types::AvatarMetadata;

use super::cache::Result;

#[async_trait(? Send)]
impl AvatarCache for IndexedDBDataCache {
    type Image = String;
    type Error = IndexedDBDataCacheError;

    async fn cache_avatar_image(
        &self,
        _jid: &BareJid,
        image: &AvatarData,
        metadata: &AvatarMetadata,
    ) -> Result<()> {
        self.db
            .set_value(
                keys::AVATAR_STORE,
                &metadata.checksum,
                image.base64().as_ref(),
            )
            .await?;
        Ok(())
    }

    async fn has_cached_avatar_image(
        &self,
        _jid: &BareJid,
        image_checksum: &ImageId,
    ) -> Result<bool> {
        let tx = self
            .db
            .transaction_on_one_with_mode(keys::AVATAR_STORE, IdbTransactionMode::Readonly)?;
        let avatar_store = tx.object_store(keys::AVATAR_STORE)?;
        let key_exists = avatar_store
            .get_key(&JsValue::from_str(image_checksum.as_ref()))?
            .await?
            .is_some();
        Ok(key_exists)
    }

    async fn cached_avatar_image(
        &self,
        jid: &BareJid,
        image_checksum: &ImageId,
    ) -> Result<Option<Self::Image>> {
        let tx = self.db.transaction_on_multi_with_mode(
            &[keys::AVATAR_METADATA_STORE, keys::AVATAR_STORE],
            IdbTransactionMode::Readonly,
        )?;

        let avatar_metadata_store = tx.object_store(keys::AVATAR_METADATA_STORE)?;
        let avatar_store = tx.object_store(keys::AVATAR_STORE)?;

        let avatar_metadata = avatar_metadata_store
            .get_value::<AvatarMetadata>(jid.to_string())
            .await?;
        let base64_data = avatar_store.get_value::<String>(image_checksum).await?;

        let (Some(avatar_metadata), Some(base64_data)) = (avatar_metadata, base64_data) else {
            return Ok(None);
        };

        let data_url = format!("data:{};base64,{}", avatar_metadata.mime_type, base64_data);

        Ok(Some(data_url))
    }

    async fn delete_all_cached_images(&self) -> Result<()> {
        self.db.clear_stores(&[keys::AVATAR_STORE]).await?;
        Ok(())
    }
}
