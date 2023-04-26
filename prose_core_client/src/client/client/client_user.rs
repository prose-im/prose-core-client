use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Instant;

use image::GenericImageView;
use tracing::{info, instrument};

use prose_core_domain::{Availability, UserProfile};

use crate::cache::{
    AvatarCache, DataCache, IMAGE_OUTPUT_FORMAT, IMAGE_OUTPUT_MIME_TYPE, MAX_IMAGE_DIMENSIONS,
};
use crate::domain_ext;
use crate::types::AvatarMetadata;

use super::Client;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    pub async fn save_profile(&self, profile: UserProfile) -> anyhow::Result<()> {
        let profile: domain_ext::UserProfile = profile.into();
        let jid = self.ctx.set_vcard(&profile).await?;
        self.ctx.publish_vcard(&profile).await?;

        self.ctx.data_cache.insert_user_profile(&jid, &profile)?;

        Ok(())
    }

    pub async fn delete_profile(&self) -> anyhow::Result<()> {
        self.ctx.unpublish_vcard().await?;
        let jid = self.ctx.delete_vcard().await?;
        self.ctx.data_cache.delete_user_profile(&jid)?;
        Ok(())
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

    #[instrument]
    pub async fn set_availability(
        &self,
        availability: Availability,
        status: Option<&str>,
    ) -> anyhow::Result<()> {
        let availability = crate::domain_ext::Availability::from(availability);
        self.ctx
            .send_presence(Some(availability.try_into()?), status)
            .await
    }
}
