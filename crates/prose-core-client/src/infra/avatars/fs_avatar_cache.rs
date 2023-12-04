// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::Result;
use async_trait::async_trait;
use base64::DecodeError;
use image::{guess_format, ImageError, ImageFormat, ImageOutputFormat};
use thiserror::Error;

use prose_xmpp::mods::AvatarData;

use crate::domain::shared::models::UserId;
use crate::domain::user_info::models::{AvatarImageId, AvatarInfo, PlatformImage};
use crate::infra::avatars::{AvatarCache, MAX_IMAGE_DIMENSIONS};

pub const IMAGE_OUTPUT_FORMAT: ImageOutputFormat = ImageOutputFormat::Jpeg(94);
pub const IMAGE_OUTPUT_MIME_TYPE: &str = "image/jpeg";

pub struct FsAvatarCache {
    path: PathBuf,
}

impl FsAvatarCache {
    pub fn new(path: &Path) -> Result<Self> {
        fs::create_dir_all(&path)?;

        Ok(FsAvatarCache {
            path: path.to_path_buf(),
        })
    }
}

#[derive(Error, Debug)]
pub enum FsAvatarCacheError {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    Image(#[from] ImageError),

    #[error(transparent)]
    Decode(#[from] DecodeError),
}

#[async_trait]
impl AvatarCache for FsAvatarCache {
    async fn cache_avatar_image(
        &self,
        jid: &UserId,
        image_data: &AvatarData,
        info: &AvatarInfo,
    ) -> Result<()> {
        let image_buf_cow = image_data.data()?;
        let image_buf = image_buf_cow.as_ref();
        let image_format =
            ImageFormat::from_mime_type(&info.mime_type).unwrap_or(guess_format(&image_buf)?);

        let img = image::load_from_memory_with_format(&image_buf, image_format)?
            .thumbnail(MAX_IMAGE_DIMENSIONS.0, MAX_IMAGE_DIMENSIONS.1);

        let output_path = self.path.join(self.filename_for(jid, &info.checksum));
        let mut output_file = std::fs::File::create(&output_path)?;

        // Sometimes we encounter e.g. rgb16 pngs and image-rs complains that the JPEG encoder
        // cannot save these, so we convert the image to rgb8.
        img.into_rgb8()
            .write_to(&mut output_file, IMAGE_OUTPUT_FORMAT)?;
        Ok(())
    }

    async fn has_cached_avatar_image(
        &self,
        jid: &UserId,
        image_checksum: &AvatarImageId,
    ) -> Result<bool> {
        let path = self.filename_for(jid, image_checksum);
        Ok(path.exists())
    }

    async fn cached_avatar_image(
        &self,
        jid: &UserId,
        image_checksum: &AvatarImageId,
    ) -> Result<Option<PlatformImage>> {
        let path = self.filename_for(jid, image_checksum);
        if path.exists() {
            return Ok(Some(path));
        }
        return Ok(None);
    }

    async fn delete_all_cached_images(&self) -> Result<()> {
        for entry in fs::read_dir(&self.path)? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => return Err(err.into()),
            };
            let metadata = entry.metadata()?;
            if metadata.is_file()
                && entry.path().extension().and_then(|ext| ext.to_str()) == Some("jpg")
            {
                fs::remove_file(entry.path())?
            }
        }
        Ok(())
    }
}

impl FsAvatarCache {
    fn filename_for(&self, jid: &UserId, image_checksum: &AvatarImageId) -> PathBuf {
        self.path
            .join(format!(
                "{}-{}.jpg",
                jid.to_string(),
                image_checksum.as_ref()
            ))
    }
}
