// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use mime::Mime;
use url::Url;
use wasm_bindgen::prelude::wasm_bindgen;

use prose_core_client::dtos;

use crate::types::{IntoJSArray, UploadHeadersArray};

#[wasm_bindgen]
pub struct UploadSlot {
    pub(crate) upload_url: Url,
    pub(crate) upload_headers: Vec<UploadHeader>,
    pub(crate) download_url: Url,
    pub(crate) media_type: Mime,
    pub(crate) file_size: u64,
    pub(crate) file_name: String,
}

#[wasm_bindgen]
impl UploadSlot {
    /// The URL where the file should be uploaded to.
    #[wasm_bindgen(getter, js_name = "uploadURL")]
    pub fn upload_url(&self) -> String {
        self.upload_url.to_string()
    }

    /// Set these headers on your PUT request when uploading.
    #[wasm_bindgen(getter, js_name = "uploadHeaders")]
    pub fn upload_headers(&self) -> UploadHeadersArray {
        self.upload_headers.iter().cloned().collect_into_js_array()
    }

    /// The URL where the file will be available after the upload.
    #[wasm_bindgen(getter, js_name = "downloadURL")]
    pub fn download_url(&self) -> String {
        self.download_url.to_string()
    }

    /// The name of the file from the initial request.
    #[wasm_bindgen(getter, js_name = "fileName")]
    pub fn file_name(&self) -> String {
        self.file_name.clone()
    }

    /// The media type of the file from the initial request.
    #[wasm_bindgen(getter, js_name = "mediaType")]
    pub fn media_type(&self) -> String {
        self.media_type.to_string()
    }

    /// The size of the file from the initial request.
    #[wasm_bindgen(getter, js_name = "fileSize")]
    pub fn file_size(&self) -> u64 {
        self.file_size
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct UploadHeader {
    name: String,
    value: String,
}

#[wasm_bindgen]
impl UploadHeader {
    /// The name of the `UploadHeader`.
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// The value of the `UploadHeader`.
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

impl From<dtos::UploadHeader> for UploadHeader {
    fn from(value: dtos::UploadHeader) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<dtos::UploadSlot> for UploadSlot {
    fn from(value: dtos::UploadSlot) -> Self {
        Self {
            upload_url: value.upload_url,
            upload_headers: value.upload_headers.into_iter().map(Into::into).collect(),
            download_url: value.download_url,
            media_type: value.media_type,
            file_size: value.file_size,
            file_name: value.file_name,
        }
    }
}
