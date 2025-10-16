// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{Mime, Url};
use prose_core_client::dtos::{
    Attachment as CoreAttachment, AttachmentType as CoreAttachmentType, Thumbnail as CoreThumbnail,
};

#[derive(uniffi::Record)]
pub struct Attachment {
    pub r#type: AttachmentType,
    pub url: Url,
    pub media_type: Mime,
    pub file_name: String,
    pub file_size: Option<u64>,
}

#[derive(uniffi::Enum)]
pub enum AttachmentType {
    Audio {
        duration: Option<u64>,
    },
    Image {
        thumbnail: Option<Thumbnail>,
    },
    Video {
        duration: Option<u64>,
        thumbnail: Option<Thumbnail>,
    },
    File,
}

#[derive(uniffi::Record)]
pub struct Thumbnail {
    pub url: Url,
    pub media_type: Mime,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl From<CoreAttachment> for Attachment {
    fn from(value: CoreAttachment) -> Self {
        Attachment {
            r#type: value.r#type.into(),
            url: value.url.into(),
            media_type: value.media_type.into(),
            file_name: value.file_name,
            file_size: value.file_size,
        }
    }
}

impl From<Attachment> for CoreAttachment {
    fn from(value: Attachment) -> Self {
        CoreAttachment {
            r#type: value.r#type.into(),
            url: value.url.into(),
            media_type: value.media_type.into(),
            file_name: value.file_name,
            file_size: value.file_size,
        }
    }
}

impl From<CoreAttachmentType> for AttachmentType {
    fn from(value: CoreAttachmentType) -> Self {
        match value {
            CoreAttachmentType::Audio { duration } => AttachmentType::Audio { duration },
            CoreAttachmentType::Image { thumbnail } => AttachmentType::Image {
                thumbnail: thumbnail.map(Into::into),
            },
            CoreAttachmentType::Video {
                duration,
                thumbnail,
            } => AttachmentType::Video {
                duration,
                thumbnail: thumbnail.map(Into::into),
            },
            CoreAttachmentType::File => AttachmentType::File,
        }
    }
}

impl From<AttachmentType> for CoreAttachmentType {
    fn from(value: AttachmentType) -> Self {
        match value {
            AttachmentType::Audio { duration } => CoreAttachmentType::Audio { duration },
            AttachmentType::Image { thumbnail } => CoreAttachmentType::Image {
                thumbnail: thumbnail.map(Into::into),
            },
            AttachmentType::Video {
                duration,
                thumbnail,
            } => CoreAttachmentType::Video {
                duration,
                thumbnail: thumbnail.map(Into::into),
            },
            AttachmentType::File => CoreAttachmentType::File,
        }
    }
}

impl From<CoreThumbnail> for Thumbnail {
    fn from(value: CoreThumbnail) -> Self {
        Thumbnail {
            url: value.url.into(),
            media_type: value.media_type.into(),
            width: value.width,
            height: value.height,
        }
    }
}

impl From<Thumbnail> for CoreThumbnail {
    fn from(value: Thumbnail) -> Self {
        CoreThumbnail {
            url: value.url.into(),
            media_type: value.media_type.into(),
            width: value.width,
            height: value.height,
        }
    }
}
