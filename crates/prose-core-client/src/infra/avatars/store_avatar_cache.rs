// // prose-core-client/prose-core-client
// //
// // Copyright: 2023, Marc Bauer <mb@nesium.com>
// // License: Mozilla Public License v2.0 (MPL v2.0)
//
// use async_trait::async_trait;
// use jid::BareJid;
// use prose_store::prelude::*;
// use prose_xmpp::mods::AvatarData;
//
// use crate::avatar_cache::AvatarCache;
// use prose_xmpp::stanza::avatar::ImageId;
//
// use crate::data_cache::indexed_db::cache::keys;
// use crate::data_cache::indexed_db::IndexedDBDataCache;
// use crate::types::AvatarMetadata;
//
// #[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
// #[async_trait]
// impl<D: Driver> AvatarCache for IndexedDBDataCache<D> {
//   type Image = String;
//   type Error = D::Error;
//
//   async fn cache_avatar_image(
//     &self,
//     _jid: &BareJid,
//     image: &AvatarData,
//     metadata: &AvatarMetadata,
//   ) -> Result<(), Self::Error> {
//     self.db
//         .set(
//           keys::AVATAR_STORE,
//           &metadata.checksum.as_ref(),
//           image.base64().as_ref(),
//         )
//         .await
//   }
//
//   async fn has_cached_avatar_image(
//     &self,
//     _jid: &BareJid,
//     image_checksum: &ImageId,
//   ) -> Result<bool, Self::Error> {
//     self.db
//         .contains_key(keys::AVATAR_STORE, image_checksum.as_ref())
//         .await
//   }
//
//   async fn cached_avatar_image(
//     &self,
//     jid: &BareJid,
//     image_checksum: &ImageId,
//   ) -> Result<Option<Self::Image>, Self::Error> {
//     let tx = self
//         .db
//         .transaction_for_reading(&[keys::AVATAR_METADATA_STORE, keys::AVATAR_STORE])
//         .await?;
//
//     let avatar_metadata = tx
//         .readable_collection(keys::AVATAR_METADATA_STORE)?
//         .get::<_, AvatarMetadata>(&jid)
//         .await?;
//     let base64_data = tx
//         .readable_collection(keys::AVATAR_STORE)?
//         .get::<_, String>(image_checksum.as_ref())
//         .await?;
//
//     let (Some(avatar_metadata), Some(base64_data)) = (avatar_metadata, base64_data) else {
//       return Ok(None);
//     };
//
//     let data_url = format!("data:{};base64,{}", avatar_metadata.mime_type, base64_data);
//
//     Ok(Some(data_url))
//   }
//
//   async fn delete_all_cached_images(&self) -> Result<(), Self::Error> {
//     self.db.truncate_collections(&[keys::AVATAR_STORE]).await?;
//     Ok(())
//   }
// }
