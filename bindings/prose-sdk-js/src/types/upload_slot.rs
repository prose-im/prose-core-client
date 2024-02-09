// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::prelude::wasm_bindgen;

use prose_core_client::dtos;

use crate::types::{IntoJSArray, UploadHeadersArray};

#[wasm_bindgen]
pub struct UploadSlot {
    upload_url: String,
    /// Set these headers on your PUT request when uploading.
    upload_headers: Vec<UploadHeader>,
    download_url: String,
}

#[wasm_bindgen]
impl UploadSlot {
    #[wasm_bindgen(getter, js_name = "uploadURL")]
    pub fn upload_url(&self) -> String {
        self.upload_url.clone()
    }

    #[wasm_bindgen(getter, js_name = "uploadHeaders")]
    pub fn upload_headers(&self) -> UploadHeadersArray {
        self.upload_headers.iter().cloned().collect_into_js_array()
    }

    #[wasm_bindgen(getter, js_name = "downloadURL")]
    pub fn download_url(&self) -> String {
        self.download_url.clone()
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
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

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
            upload_url: value.upload_url.to_string(),
            upload_headers: value.upload_headers.into_iter().map(Into::into).collect(),
            download_url: value.download_url.to_string(),
        }
    }
}
