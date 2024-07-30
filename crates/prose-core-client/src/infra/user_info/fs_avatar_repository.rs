// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::path::{Path, PathBuf};
use std::{fs, io};

use anyhow::Result;
use async_trait::async_trait;
use base64::DecodeError;
use image::codecs::jpeg::JpegEncoder;
use image::{guess_format, ImageError, ImageFormat};
use thiserror::Error;

use prose_xmpp::mods::AvatarData;

use crate::domain::shared::models::{AccountId, AvatarId, ParticipantIdRef};
use crate::domain::user_info::models::{AvatarInfo, PlatformImage};
use crate::domain::user_info::repos::AvatarRepository;
use crate::dtos::{Avatar, AvatarSource};

use super::MAX_IMAGE_DIMENSIONS;

pub struct FsAvatarRepository {
    path: PathBuf,
}

impl FsAvatarRepository {
    pub fn new(path: &Path) -> Result<Self> {
        fs::create_dir_all(&path)?;

        Ok(FsAvatarRepository {
            path: path.to_path_buf(),
        })
    }
}

#[derive(Error, Debug)]
pub enum FsAvatarRepositoryError {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    Image(#[from] ImageError),

    #[error(transparent)]
    Decode(#[from] DecodeError),
}

#[async_trait]
impl AvatarRepository for FsAvatarRepository {
    async fn set(
        &self,
        _account: &AccountId,
        participant_id: ParticipantIdRef<'_>,
        info: &AvatarInfo,
        image_data: &AvatarData,
    ) -> Result<()> {
        let image_buf_cow = image_data.data()?;
        let image_buf = image_buf_cow.as_ref();
        let image_format =
            ImageFormat::from_mime_type(&info.mime_type).unwrap_or(guess_format(&image_buf)?);

        let img = image::load_from_memory_with_format(&image_buf, image_format)?
            .thumbnail(MAX_IMAGE_DIMENSIONS.0, MAX_IMAGE_DIMENSIONS.1);

        let output_path = self
            .path
            .join(self.filename_for(participant_id, &info.checksum));

        if let Some(parent_dir) = output_path.parent() {
            if !parent_dir.exists() {
                fs::create_dir(parent_dir)?;
            }
        }

        let mut output_file = fs::File::create(&output_path)?;
        let encoder = JpegEncoder::new_with_quality(&mut output_file, 94);

        // Sometimes we encounter e.g. rgb16 pngs and image-rs complains that the JPEG encoder
        // cannot save these, so we convert the image to rgb8.
        img.into_rgb8().write_with_encoder(encoder)?;
        Ok(())
    }

    async fn get(&self, _account: &AccountId, avatar: &Avatar) -> Result<Option<PlatformImage>> {
        let participant_id = match &avatar.source {
            AvatarSource::Pep { owner, .. } => ParticipantIdRef::User(owner),
            AvatarSource::Vcard { owner } => owner.to_ref(),
        };

        let path = self.filename_for(participant_id, &avatar.id);
        if path.exists() {
            return Ok(Some(path));
        }
        return Ok(None);
    }

    async fn clear_cache(&self, _account: &AccountId) -> Result<()> {
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

impl FsAvatarRepository {
    fn filename_for(
        &self,
        participant_id: ParticipantIdRef<'_>,
        image_checksum: &AvatarId,
    ) -> PathBuf {
        match participant_id {
            ParticipantIdRef::User(id) => self.path.join(format!("{id}-{image_checksum}.jpg")),
            ParticipantIdRef::Occupant(id) => self
                .path
                .join(id.muc_id().as_ref().to_string())
                .join(format!("{}-{image_checksum}.jpg", id.nickname())),
        }
    }
}
