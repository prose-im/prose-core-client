use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Error, Result};
use image::{DynamicImage, ImageOutputFormat};
use jid::BareJid;

use prose_xmpp::stanza::avatar;

use crate::avatar_cache::AvatarCache;
use crate::types::AvatarMetadata;

pub const IMAGE_OUTPUT_FORMAT: ImageOutputFormat = ImageOutputFormat::Jpeg(80);

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

impl AvatarCache for FsAvatarCache {
    fn cache_avatar_image(
        &self,
        jid: &BareJid,
        image: DynamicImage,
        metadata: &AvatarMetadata,
    ) -> Result<PathBuf> {
        let output_path = self.path.join(self.filename_for(jid, &metadata.checksum));
        let mut output_file = std::fs::File::create(&output_path)?;

        // Sometimes we encounter e.g. rgb16 pngs and image-rs complains that the JPEG encoder
        // cannot save these, so we convert the image to rgb8.
        image
            .into_rgb8()
            .write_to(&mut output_file, IMAGE_OUTPUT_FORMAT)?;
        Ok(output_path)
    }

    fn cached_avatar_image_url(
        &self,
        jid: &BareJid,
        image_checksum: &avatar::ImageId,
    ) -> Option<PathBuf> {
        let path = self.filename_for(jid, image_checksum);
        if path.exists() {
            return Some(path);
        }
        return None;
    }

    fn delete_all_cached_images(&self) -> Result<()> {
        for entry in fs::read_dir(&self.path)? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) => return Err(Error::new(err)),
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
    fn filename_for(&self, jid: &BareJid, image_checksum: &avatar::ImageId) -> PathBuf {
        self.path.join(format!(
            "{}-{}.jpg",
            jid.to_string(),
            image_checksum.as_ref()
        ))
    }
}
