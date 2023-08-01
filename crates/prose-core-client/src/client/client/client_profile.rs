use std::fmt::Debug;
use std::str::FromStr;

use anyhow::Result;
use jid::BareJid;
use tracing::{debug, instrument};
use xmpp_parsers::hashes::Sha1HexAttribute;

use prose_xmpp::mods::{AvatarData, Profile};
use prose_xmpp::stanza::VCard4;

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::{AvatarMetadata, UserProfile};
use crate::CachePolicy;

use super::Client;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    pub async fn save_profile(&self, user_profile: UserProfile) -> Result<()> {
        let profile = self.client.get_mod::<Profile>();

        let vcard = VCard4::from(user_profile.clone());

        profile.set_vcard(vcard.clone()).await?;
        profile.publish_vcard(vcard).await?;

        self.inner
            .data_cache
            .insert_user_profile(&self.connected_jid()?.into(), &user_profile)
            .await?;

        Ok(())
    }

    pub async fn delete_profile(&self) -> Result<()> {
        let profile = self.client.get_mod::<Profile>();
        profile.unpublish_vcard().await?;
        profile.delete_vcard().await?;
        self.inner
            .data_cache
            .delete_user_profile(&self.connected_jid()?.into())
            .await?;
        Ok(())
    }

    #[instrument]
    pub async fn load_avatar(
        &self,
        from: impl Into<BareJid> + Debug,
        cache_policy: CachePolicy,
    ) -> Result<Option<A::Image>> {
        let from = from.into();

        let Some(metadata) = self.load_avatar_metadata(&from, cache_policy).await? else {
            return Ok(None);
        };

        self.load_and_cache_avatar_image(&from, &metadata, cache_policy)
            .await?;

        let image = self
            .inner
            .avatar_cache
            .cached_avatar_image(&from, &metadata.checksum)
            .await?;

        Ok(image)
    }

    pub async fn save_avatar(
        &self,
        image_data: impl AsRef<[u8]>,
        width: Option<u32>,
        height: Option<u32>,
        mime_type: impl AsRef<str>,
    ) -> Result<()> {
        let image_data_len = image_data.as_ref().len();
        let image_data = AvatarData::Data(image_data.as_ref().to_vec());

        let metadata = AvatarMetadata::new(
            mime_type.as_ref().to_string(),
            image_data.generate_sha1_checksum()?,
            width,
            height,
        );

        debug!("Uploading avatar…");
        let profile = self.client.get_mod::<Profile>();

        profile
            .set_avatar_image(&metadata.checksum, image_data.base64())
            .await?;

        debug!("Uploading avatar metadata…");
        profile
            .set_avatar_metadata(
                image_data_len,
                &metadata.checksum,
                mime_type.as_ref(),
                metadata.width,
                metadata.height,
            )
            .await?;

        let jid = jid::BareJid::from(self.connected_jid()?);

        debug!("Caching avatar metadata");
        self.inner
            .data_cache
            .insert_avatar_metadata(&jid, &metadata)
            .await?;

        debug!("Caching image locally…");
        let path = self
            .inner
            .avatar_cache
            .cache_avatar_image(&jid, &image_data, &metadata)
            .await?;

        Ok(path)
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[instrument]
    pub async fn save_avatar_from_url(&self, image_path: &std::path::Path) -> Result<()> {
        use crate::avatar_cache::fs_avatar_cache::IMAGE_OUTPUT_FORMAT;
        use crate::avatar_cache::fs_avatar_cache::IMAGE_OUTPUT_MIME_TYPE;
        use crate::avatar_cache::MAX_IMAGE_DIMENSIONS;
        use image::GenericImageView;
        use std::io::Cursor;
        use std::time::Instant;

        let now = Instant::now();
        debug!("Opening & resizing image at {:?}…", image_path);

        let img =
            image::open(image_path)?.thumbnail(MAX_IMAGE_DIMENSIONS.0, MAX_IMAGE_DIMENSIONS.1);
        debug!(
            "Opening image & resizing finished after {:.2?}",
            now.elapsed()
        );

        let mut image_data = Vec::new();
        img.write_to(&mut Cursor::new(&mut image_data), IMAGE_OUTPUT_FORMAT)?;

        self.save_avatar(
            image_data,
            Some(img.dimensions().0),
            Some(img.dimensions().1),
            IMAGE_OUTPUT_MIME_TYPE,
        )
        .await
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub(super) async fn load_and_cache_avatar_image(
        &self,
        from: &BareJid,
        metadata: &AvatarMetadata,
        cache_policy: CachePolicy,
    ) -> Result<()> {
        if cache_policy != CachePolicy::ReloadIgnoringCacheData {
            if self
                .inner
                .avatar_cache
                .has_cached_avatar_image(&from, &metadata.checksum)
                .await?
            {
                debug!("Found cached image for {}", from);
                return Ok(());
            }
        }

        if cache_policy == CachePolicy::ReturnCacheDataDontLoad {
            return Ok(());
        }

        let profile = self.client.get_mod::<Profile>();

        let Some(avatar_data) = profile
            .load_avatar_image(
                from.clone(),
                &Sha1HexAttribute::from_str(&metadata.checksum.as_ref())?,
            )
            .await?
        else {
            return Ok(());
        };

        self.inner
            .avatar_cache
            .cache_avatar_image(from, &avatar_data, &metadata)
            .await?;

        Ok(())
    }

    #[instrument]
    async fn load_avatar_metadata(
        &self,
        from: &BareJid,
        cache_policy: CachePolicy,
    ) -> Result<Option<AvatarMetadata>> {
        if cache_policy != CachePolicy::ReloadIgnoringCacheData {
            if let Some(metadata) = self.inner.data_cache.load_avatar_metadata(from).await? {
                return Ok(Some(metadata.into()));
            }
        }

        if cache_policy == CachePolicy::ReturnCacheDataDontLoad {
            return Ok(None);
        }

        let profile = self.client.get_mod::<Profile>();
        let metadata = profile
            .load_latest_avatar_metadata(from.clone())
            .await?
            .map(Into::into);

        let Some(metadata) = metadata else {
            return Ok(None);
        };
        self.inner
            .data_cache
            .insert_avatar_metadata(from, &metadata)
            .await?;
        Ok(Some(metadata))
    }
}
