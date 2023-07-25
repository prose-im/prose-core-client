use anyhow::Result;
use tracing::instrument;

use prose_domain::UserProfile;
use prose_xmpp::mods::Profile;

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::domain_ext;

use super::Client;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    pub async fn save_profile(&self, profile: UserProfile) -> Result<()> {
        let user_profile: domain_ext::UserProfile = profile.into();
        let profile = self.client.get_mod::<Profile>();

        profile.publish_vcard(user_profile.clone().into()).await?;

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

    #[cfg(feature = "native-app")]
    #[instrument]
    pub async fn save_avatar(&self, image_path: &std::path::Path) -> Result<std::path::PathBuf> {
        use crate::avatar_cache::fs_avatar_cache::IMAGE_OUTPUT_FORMAT;
        use crate::avatar_cache::{IMAGE_OUTPUT_MIME_TYPE, MAX_IMAGE_DIMENSIONS};
        use crate::types::AvatarMetadata;
        use image::GenericImageView;
        use std::io::Cursor;
        use std::time::Instant;

        let now = Instant::now();
        tracing::info!("Opening & resizing image at {:?}…", image_path);

        let img =
            image::open(image_path)?.thumbnail(MAX_IMAGE_DIMENSIONS.0, MAX_IMAGE_DIMENSIONS.1);
        tracing::info!(
            "Opening image & resizing finished after {:.2?}",
            now.elapsed()
        );

        let mut image_data = Vec::new();
        img.write_to(&mut Cursor::new(&mut image_data), IMAGE_OUTPUT_FORMAT)?;

        let metadata = AvatarMetadata::new(
            IMAGE_OUTPUT_MIME_TYPE,
            AvatarMetadata::generate_sha1_checksum(&image_data).into(),
            Some(img.dimensions().0),
            Some(img.dimensions().1),
        );

        tracing::info!("Uploading avatar…");
        let profile = self.client.get_mod::<Profile>();

        profile
            .set_avatar_image(
                &metadata.checksum,
                AvatarMetadata::encode_image_data(&image_data),
            )
            .await?;

        tracing::info!("Uploading avatar metadata…");
        profile
            .set_avatar_metadata(
                image_data.len(),
                &metadata.checksum,
                IMAGE_OUTPUT_MIME_TYPE,
                metadata.width,
                metadata.height,
            )
            .await?;

        let jid = jid::BareJid::from(self.connected_jid()?);

        tracing::info!("Caching avatar metadata");
        self.inner
            .data_cache
            .insert_avatar_metadata(&jid, &metadata)
            .await?;

        tracing::info!("Caching image locally…");
        let path = self
            .inner
            .avatar_cache
            .cache_avatar_image(&jid, img, &metadata)?;

        Ok(path)
    }
}
