// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::anyhow;
use mime::Mime;
use url::Url;
use wasm_bindgen::prelude::*;

use prose_core_client::dtos;

use crate::types::UploadSlot;

#[wasm_bindgen]
#[derive(Clone, Copy)]
/// The type of attachment. Derived from the attachment's media type.
pub enum AttachmentType {
    Image = 0,
    Audio = 1,
    Video = 2,
    File = 3,
}

impl TryFrom<u32> for AttachmentType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Image),
            1 => Ok(Self::Audio),
            2 => Ok(Self::Video),
            3 => Ok(Self::File),
            _ => Err(anyhow!("Invalid AttachmentType '{value}'.")),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
// An attachment to a message.
pub struct Attachment {
    pub(crate) r#type: AttachmentType,
    pub(crate) metadata: AttachmentMetadata,
    pub(crate) duration: Option<u64>,
    pub(crate) thumbnail: Option<Thumbnail>,
}

#[wasm_bindgen]
impl Attachment {
    /// Creates an attachment with an image. Provide a thumbnail for the inline preview.
    #[wasm_bindgen(js_name = "imageAttachment")]
    pub fn image_attachment(metadata: AttachmentMetadata, thumbnail: Thumbnail) -> Self {
        Self {
            r#type: AttachmentType::Image,
            metadata,
            duration: None,
            thumbnail: Some(thumbnail),
        }
    }

    /// Creates an attachment with an audio file. Provide the duration (in seconds) of the
    /// audio clip for the preview.
    #[wasm_bindgen(js_name = "audioAttachment")]
    pub fn audio_attachment(metadata: AttachmentMetadata, duration: u64) -> Self {
        Self {
            r#type: AttachmentType::Audio,
            metadata,
            duration: Some(duration),
            thumbnail: None,
        }
    }

    /// Creates an attachment with a video. Provide the duration of the video and a thumbnail
    /// for the inline preview.
    #[wasm_bindgen(js_name = "videoAttachment")]
    pub fn video_attachment(
        metadata: AttachmentMetadata,
        duration: u64,
        thumbnail: Thumbnail,
    ) -> Self {
        Self {
            r#type: AttachmentType::Audio,
            metadata,
            duration: Some(duration),
            thumbnail: Some(thumbnail),
        }
    }

    /// Creates an attachment with a generic file.
    #[wasm_bindgen(js_name = "fileAttachment")]
    pub fn file_attachment(metadata: AttachmentMetadata) -> Self {
        Self {
            r#type: AttachmentType::Audio,
            metadata,
            duration: None,
            thumbnail: None,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
/// A thumbnail for an attachment.
pub struct Thumbnail {
    pub(crate) url: Url,
    pub(crate) media_type: Mime,
    pub(crate) width: Option<u32>,
    pub(crate) height: Option<u32>,
}

#[wasm_bindgen]
impl Attachment {
    /// The URL of the attachment.
    #[wasm_bindgen(getter)]
    pub fn url(&self) -> String {
        self.metadata.url.to_string()
    }

    /// The type of the attachment.
    #[wasm_bindgen(getter, js_name = "type")]
    pub fn r#type(&self) -> AttachmentType {
        self.r#type
    }

    /// The duration of the attachment in seconds. Only available (but not necessarily) if `type`
    /// is `AttachmentType.Audio`.
    #[wasm_bindgen(getter)]
    pub fn duration(&self) -> Option<u64> {
        self.duration.clone()
    }

    /// A thumbnail for inline preview of the attachment. Only available (but not necessarily)
    /// if `type` is `AttachmentType.Image` or `AttachmentType.Video`.
    #[wasm_bindgen(getter)]
    pub fn thumbnail(&self) -> Option<Thumbnail> {
        self.thumbnail.clone()
    }

    /// The media type of the attachment.
    #[wasm_bindgen(getter, js_name = "mediaType")]
    pub fn media_type(&self) -> String {
        self.metadata.media_type.to_string()
    }

    /// The file name of the attachment.
    #[wasm_bindgen(getter, js_name = "fileName")]
    pub fn file_name(&self) -> String {
        self.metadata.file_name.clone()
    }

    /// The size of the attachment in bytes (if available).
    #[wasm_bindgen(getter, js_name = "fileSize")]
    pub fn file_size(&self) -> Option<u64> {
        self.metadata.file_size.clone()
    }
}

#[wasm_bindgen]
impl Thumbnail {
    /// Instantiates a new `Thumbnail`. If you have an `UploadSlot` available, prefer to
    /// use `Thumbnail.fromSlot` instead.
    #[wasm_bindgen(constructor)]
    pub fn new(url: String, media_type: String, width: u32, height: u32) -> Self {
        Self {
            url: url
                .parse()
                .expect("Received invalid URL '{url}' in MessageAttachmentThumbnail constructor."),
            media_type: media_type.parse()
                .expect("Received invalid media type '{media_type}' in MessageAttachmentThumbnail constructor."),
            width: Some(width),
            height: Some(height)
        }
    }

    /// Instantiates a new `Thumbnail` from an `UploadSlot` and a given `width` and `height`.
    #[wasm_bindgen(js_name = "fromSlot")]
    pub fn from_slot(slot: UploadSlot, width: u32, height: u32) -> Self {
        Self {
            url: slot.download_url,
            media_type: slot.media_type,
            width: Some(width),
            height: Some(height),
        }
    }

    /// The URL of the thumbnail.
    #[wasm_bindgen(getter)]
    pub fn url(&self) -> String {
        self.url.to_string()
    }

    /// The media type of the thumbnail.
    #[wasm_bindgen(getter, js_name = "mediaType")]
    pub fn media_type(&self) -> String {
        self.media_type.to_string()
    }

    /// The width of the thumbnail in pixels.
    #[wasm_bindgen(getter, js_name = "width")]
    pub fn width(&self) -> Option<u32> {
        self.width.clone()
    }

    /// The height of the thumbnail in pixels.
    #[wasm_bindgen(getter, js_name = "height")]
    pub fn height(&self) -> Option<u32> {
        self.height.clone()
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct AttachmentMetadata {
    pub(crate) url: Url,
    pub(crate) media_type: Mime,
    pub(crate) file_name: String,
    pub(crate) file_size: Option<u64>,
}

#[wasm_bindgen]
impl AttachmentMetadata {
    /// Instantiates a new `AttachmentMetadata`. If you have an `UploadSlot` available, prefer to
    /// use `AttachmentMetadata.fromSlot` instead.
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str, media_type: &str, file_name: &str, file_size: u64) -> Self {
        Self {
            url: url
                .parse()
                .expect("Received invalid URL in AttachmentMetadata constructor."),
            media_type: media_type
                .parse()
                .expect("Received invalid media type in AttachmentMetadata constructor."),
            file_name: file_name.to_string(),
            file_size: Some(file_size),
        }
    }

    /// Instantiates a new `AttachmentMetadata` from an `UploadSlot`.
    #[wasm_bindgen(js_name = "fromSlot")]
    pub fn from_slot(slot: UploadSlot) -> Self {
        Self {
            url: slot.download_url,
            media_type: slot.media_type,
            file_name: slot.file_name,
            file_size: Some(slot.file_size),
        }
    }
}

impl From<dtos::Attachment> for Attachment {
    fn from(value: dtos::Attachment) -> Self {
        let (r#type, duration, thumbnail) = match value.r#type {
            dtos::AttachmentType::Audio { duration } => (AttachmentType::Audio, duration, None),
            dtos::AttachmentType::Image { thumbnail } => (AttachmentType::Image, None, thumbnail),
            dtos::AttachmentType::Video {
                duration,
                thumbnail,
            } => (AttachmentType::Video, duration, thumbnail),
            dtos::AttachmentType::File => (AttachmentType::File, None, None),
        };

        Self {
            r#type,
            metadata: AttachmentMetadata {
                url: value.url,
                media_type: value.media_type,
                file_name: value.file_name,
                file_size: value.file_size,
            },
            duration,
            thumbnail: thumbnail.map(Into::into),
        }
    }
}

impl From<Attachment> for dtos::Attachment {
    fn from(value: Attachment) -> Self {
        let kind = match value.r#type {
            AttachmentType::Image => dtos::AttachmentType::Image {
                thumbnail: value.thumbnail.map(Into::into),
            },
            AttachmentType::Audio => dtos::AttachmentType::Audio {
                duration: value.duration,
            },
            AttachmentType::Video => dtos::AttachmentType::Video {
                duration: value.duration,
                thumbnail: value.thumbnail.map(Into::into),
            },
            AttachmentType::File => dtos::AttachmentType::File,
        };

        Self {
            r#type: kind,
            url: value.metadata.url,
            media_type: value.metadata.media_type,
            file_name: value.metadata.file_name,
            file_size: value.metadata.file_size,
        }
    }
}

impl From<dtos::Thumbnail> for Thumbnail {
    fn from(value: dtos::Thumbnail) -> Self {
        Self {
            url: value.url,
            media_type: value.media_type,
            width: value.width,
            height: value.height,
        }
    }
}

impl From<Thumbnail> for dtos::Thumbnail {
    fn from(value: Thumbnail) -> Self {
        Self {
            url: value.url,
            media_type: value.media_type,
            width: value.width,
            height: value.height,
        }
    }
}
