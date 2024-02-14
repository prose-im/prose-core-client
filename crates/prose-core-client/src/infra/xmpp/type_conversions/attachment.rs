// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::path::Path;

use anyhow::anyhow;
use mime::Mime;
use sha1::{Digest, Sha1};
use url::Url;

use prose_xmpp::stanza::media_sharing::{File, MediaShare, OOB};
use prose_xmpp::stanza::references::Reference;

use crate::domain::messaging::models::{Attachment, AttachmentType};
use crate::infra::xmpp::util::{FileExt, MediaShareExt};

impl From<Attachment> for OOB {
    fn from(value: Attachment) -> Self {
        OOB {
            url: value.url.to_string(),
            desc: None,
        }
    }
}

impl TryFrom<OOB> for Attachment {
    type Error = anyhow::Error;

    fn try_from(value: OOB) -> Result<Self, Self::Error> {
        Ok(Attachment::from(value.url.parse::<Url>()?))
    }
}

impl TryFrom<MediaShare> for Attachment {
    type Error = anyhow::Error;

    fn try_from(value: MediaShare) -> Result<Self, Self::Error> {
        let Some(file_url) = value.first_http_source_url() else {
            return Err(anyhow!(
                "Couldn't find http(s) reference in media-sharing element."
            ));
        };

        let media_type = value.file.media_type.parse::<Mime>()?;

        let kind = match media_type.type_() {
            mime::IMAGE => AttachmentType::Image {
                thumbnail: value.file.best_thumbnail_representation(),
            },
            mime::AUDIO => AttachmentType::Audio {
                duration: value.file.duration,
            },
            mime::VIDEO => AttachmentType::Video {
                duration: value.file.duration,
                thumbnail: value.file.best_thumbnail_representation(),
            },
            _ => AttachmentType::File,
        };

        let file_name = value.file.name.unwrap_or(file_url.file_name_or_hash());

        Ok(Attachment {
            r#type: kind,
            url: file_url,
            media_type,
            file_name,
            file_size: Some(value.file.size),
        })
    }
}

impl From<Url> for Attachment {
    fn from(value: Url) -> Self {
        let media_type =
            mime_guess::from_path(Path::new(value.path())).first_or(mime::APPLICATION_OCTET_STREAM);

        let kind = match media_type.type_() {
            mime::IMAGE => AttachmentType::Image { thumbnail: None },
            mime::AUDIO => AttachmentType::Audio { duration: None },
            mime::VIDEO => AttachmentType::Video {
                duration: None,
                thumbnail: None,
            },
            _ => AttachmentType::File,
        };

        let file_name = value.file_name_or_hash();

        Attachment {
            r#type: kind,
            url: value,
            media_type,
            file_name,
            file_size: None,
        }
    }
}

impl From<Attachment> for MediaShare {
    fn from(value: Attachment) -> Self {
        let mut share = MediaShare {
            file: File {
                media_type: value.media_type.to_string(),
                name: Some(value.file_name),
                size: value.file_size.unwrap_or(0),
                desc: None,
                duration: None,
                hashes: vec![],
                thumbnails: vec![],
            },
            sources: vec![Reference::data_reference(value.url.to_string())],
        };

        match value.r#type {
            AttachmentType::Audio { duration } => share.file.duration = duration,
            AttachmentType::Image { thumbnail } => {
                if let Some(thumbnail) = thumbnail {
                    share.file.thumbnails.push(thumbnail.into())
                }
            }
            AttachmentType::Video {
                thumbnail,
                duration,
            } => {
                share.file.duration = duration;
                if let Some(thumbnail) = thumbnail {
                    share.file.thumbnails.push(thumbnail.into())
                }
            }
            AttachmentType::File => {}
        }

        share
    }
}

trait UrlExt {
    fn file_name_or_hash(&self) -> String;
}

impl UrlExt for Url {
    fn file_name_or_hash(&self) -> String {
        let path = Path::new(self.path());

        let mut file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .map(ToString::to_string)
            .unwrap_or_else(|| {
                let mut hasher = Sha1::new();
                hasher.update(self.to_string());
                format!("{:x}", hasher.finalize())
            });

        if path.extension().is_none() {
            file_name.push_str(".bin")
        }

        file_name
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_attachment_from_url() -> Result<()> {
        assert_eq!(
            Attachment::from(Url::parse("https://www.google.com")?),
            Attachment {
                r#type: AttachmentType::File,
                url: Url::parse("https://www.google.com")?,
                media_type: mime::APPLICATION_OCTET_STREAM,
                // SHA1 of 'https://www.google.com/'
                file_name: "595c3cce2409a55c13076f1bac5edee529fc2e58.bin".to_string(),
                file_size: None,
            }
        );

        assert_eq!(
            Attachment::from(Url::parse("https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.jpg")?),
            Attachment {
                r#type: AttachmentType::Image { thumbnail: None },
                url: Url::parse("https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.jpg")?,
                media_type: mime::IMAGE_JPEG,
                file_name: "164492440299900_1vb3qj9.jpg".to_string(),
                file_size: None,
            }
        );

        assert_eq!(
            Attachment::from(Url::parse("https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.mp3")?),
            Attachment {
                r#type: AttachmentType::Audio { duration: None },
                url: Url::parse("https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.mp3")?,
                media_type: "audio/mpeg".parse()?,
                file_name: "164492440299900_1vb3qj9.mp3".to_string(),
                file_size: None,
            }
        );

        assert_eq!(
            Attachment::from(Url::parse("https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.mp4")?),
            Attachment {
                r#type: AttachmentType::Video { duration: None, thumbnail: None },
                url: Url::parse("https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.mp4")?,
                media_type: "video/mp4".parse()?,
                file_name: "164492440299900_1vb3qj9.mp4".to_string(),
                file_size: None,
            }
        );

        assert_eq!(
            Attachment::from(Url::parse("https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9")?),
            Attachment {
                r#type: AttachmentType::File,
                url: Url::parse("https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9")?,
                media_type: mime::APPLICATION_OCTET_STREAM,
                file_name: "164492440299900_1vb3qj9.bin".to_string(),
                file_size: None,
            }
        );

        Ok(())
    }
}
