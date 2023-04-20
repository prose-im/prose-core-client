use std::fmt::Debug;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Instant;

use image::GenericImageView;
use jid::{BareJid, Jid};
use microtype::Microtype;
use tracing::{info, instrument};

use prose_core_domain::UserProfile;

use crate::cache::{
    AvatarCache, DataCache, IMAGE_OUTPUT_FORMAT, IMAGE_OUTPUT_MIME_TYPE, MAX_IMAGE_DIMENSIONS,
};
use crate::types::AvatarMetadata;
use crate::{domain_ext, CachePolicy};

use super::Client;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    pub async fn load_profile(
        &self,
        from: impl Into<BareJid> + Debug,
        cache_policy: CachePolicy,
    ) -> anyhow::Result<UserProfile> {
        let from = from.into();

        if cache_policy != CachePolicy::ReloadIgnoringCacheData {
            if let Some(cached_profile) = self.ctx.data_cache.load_user_profile(&from)? {
                info!("Found cached profile for {}", from);
                return Ok(cached_profile);
            }
        }

        if cache_policy == CachePolicy::ReturnCacheDataDontLoad {
            return Ok(UserProfile::default());
        }

        let Some(profile) = self.ctx.load_vcard(&from).await? else {
          return Ok(UserProfile::default())
        };

        self.ctx.data_cache.insert_user_profile(&from, &profile)?;
        Ok(profile.into_inner())
    }

    #[instrument]
    pub async fn save_profile(&self, profile: UserProfile) -> anyhow::Result<()> {
        let profile: domain_ext::UserProfile = profile.into();
        let jid = self.ctx.set_vcard(&profile).await?;
        self.ctx.publish_vcard(&profile).await?;

        self.ctx.data_cache.insert_user_profile(&jid, &profile)?;

        Ok(())
    }

    #[instrument]
    pub async fn load_avatar(
        &self,
        from: impl Into<Jid> + Debug,
        cache_policy: CachePolicy,
    ) -> anyhow::Result<Option<PathBuf>> {
        let jid = BareJid::from(from.into());

        let Some(metadata) = self.load_avatar_metadata(&jid, cache_policy).await? else {
      return Ok(None)
    };

        self.ctx
            .load_and_cache_avatar_image(&jid, &metadata, cache_policy)
            .await
    }

    #[instrument]
    pub async fn save_avatar(&self, image_path: &Path) -> anyhow::Result<PathBuf> {
        let now = Instant::now();
        info!("Opening & resizing image at {:?}…", image_path);

        let img =
            image::open(image_path)?.thumbnail(MAX_IMAGE_DIMENSIONS.0, MAX_IMAGE_DIMENSIONS.1);
        info!(
            "Opening image & resizing finished after {:.2?}",
            now.elapsed()
        );

        let mut image_data = Vec::new();
        img.write_to(&mut Cursor::new(&mut image_data), IMAGE_OUTPUT_FORMAT)?;

        let metadata = AvatarMetadata::new(
            IMAGE_OUTPUT_MIME_TYPE,
            AvatarMetadata::generate_sha1_checksum(&image_data).into(),
            img.dimensions().0,
            img.dimensions().1,
        );

        info!("Uploading avatar…");
        self.ctx
            .set_avatar_image(&metadata.checksum, &image_data)
            .await?;

        info!("Uploading avatar metadata…");
        let jid = self
            .ctx
            .set_avatar_metadata(
                image_data.len(),
                &metadata.checksum,
                metadata.width,
                metadata.height,
            )
            .await?;

        info!("Caching avatar metadata");
        self.ctx
            .data_cache
            .insert_avatar_metadata(&jid, &metadata)?;

        info!("Caching image locally…");
        let path = self
            .ctx
            .avatar_cache
            .cache_avatar_image(&jid, img, &metadata)?;

        Ok(path)
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    async fn load_avatar_metadata(
        &self,
        from: &BareJid,
        cache_policy: CachePolicy,
    ) -> anyhow::Result<Option<AvatarMetadata>> {
        if cache_policy != CachePolicy::ReloadIgnoringCacheData {
            if let Some(metadata) = self.ctx.data_cache.load_avatar_metadata(from)? {
                return Ok(Some(metadata));
            }
        }

        if cache_policy == CachePolicy::ReturnCacheDataDontLoad {
            return Ok(None);
        }

        let Some(metadata) = self.ctx.load_latest_avatar_metadata(from).await? else {
      return Ok(None)
    };
        self.ctx
            .data_cache
            .insert_avatar_metadata(from, &metadata)?;
        Ok(Some(metadata))
    }
}
