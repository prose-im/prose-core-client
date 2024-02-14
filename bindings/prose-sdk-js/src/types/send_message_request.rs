// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::anyhow;
use js_sys::{BigInt, Reflect};
use mime::Mime;
use tracing::error;
use url::Url;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};

use prose_core_client::dtos;

use crate::types::attachment::{AttachmentMetadata, AttachmentType};
use crate::types::{Attachment, AttachmentsArray, IntoJSArray, Thumbnail};

#[wasm_bindgen]
pub struct SendMessageRequest {
    /// The body of the message.
    body: Option<String>,
    /// The URLs of the files to attach to the message.
    attachments: Vec<Attachment>,
}

#[wasm_bindgen]
impl SendMessageRequest {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            body: None,
            attachments: vec![],
        }
    }

    #[wasm_bindgen(getter)]
    pub fn body(&self) -> Option<String> {
        self.body.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_body(&mut self, body: Option<String>) {
        self.body = body;
    }

    #[wasm_bindgen(getter)]
    pub fn attachments(&self) -> AttachmentsArray {
        self.attachments.iter().cloned().collect_into_js_array()
    }

    #[wasm_bindgen(setter)]
    pub fn set_attachments(&mut self, attachments: AttachmentsArray) {
        let js_val: &JsValue = attachments.as_ref();
        let array: Option<&js_sys::Array> = js_val.dyn_ref();

        let Some(array) = array else {
            error!("Tried to assign a non-Array to 'attachments' of 'SendMessageRequest'.");
            return;
        };

        let Ok(length) = usize::try_from(array.length()) else {
            error!("Could not determine the length of the attachments array.");
            return;
        };

        let mut typed_array = Vec::<Attachment>::with_capacity(length);
        for js in array.iter() {
            let obj = match js.dyn_into::<js_sys::Object>() {
                Ok(obj) => obj,
                Err(err) => {
                    error!("Failed to parse attachment. {:?}", err);
                    return;
                }
            };

            let attachment = match Attachment::try_from(obj) {
                Ok(attachment) => attachment,
                Err(err) => {
                    error!("Failed to parse attachment. {}", err.to_string());
                    return;
                }
            };
            typed_array.push(attachment);
        }

        self.attachments = typed_array;
    }
}

impl TryFrom<SendMessageRequest> for dtos::SendMessageRequest {
    type Error = anyhow::Error;

    fn try_from(value: SendMessageRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            body: value.body,
            attachments: value
                .attachments
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<js_sys::Object> for Attachment {
    type Error = anyhow::Error;

    fn try_from(value: js_sys::Object) -> Result<Self, Self::Error> {
        let kind = AttachmentType::try_from(
            Reflect::get(&value, &JsValue::from_str("type"))
                .ok()
                .and_then(|value| value.as_f64())
                .ok_or_else(|| anyhow!("type is not a Number"))? as u32,
        )?;

        let metadata = AttachmentMetadata::try_from(value.clone())?;

        let thumbnail = 'thumbnail: {
            let Some(js_value) = Reflect::get(&value, &JsValue::from_str("thumbnail")).ok() else {
                break 'thumbnail None;
            };

            if js_value.is_undefined() || js_value.is_null() {
                break 'thumbnail None;
            }

            if !js_value.is_object() {
                return Err(anyhow!("thumbnail is not an object"));
            }

            let Ok(value) = js_value.dyn_into::<js_sys::Object>() else {
                return Err(anyhow!("Failed to cast thumbnail to an object"));
            };

            Some(Thumbnail::try_from(value)?)
        };

        let duration = Reflect::get(&value, &JsValue::from_str("duration"))
            .ok()
            .and_then(|value| {
                if value.is_null() || value.is_undefined() {
                    return None;
                }
                Some(value)
            })
            .map(|value| {
                u64::try_from(BigInt::from(value)).map_err(|_| anyhow!("Could not parse duration"))
            })
            .transpose()?;

        Ok(Attachment {
            r#type: kind,
            metadata,
            duration,
            thumbnail,
        })
    }
}

impl TryFrom<js_sys::Object> for AttachmentMetadata {
    type Error = anyhow::Error;

    fn try_from(value: js_sys::Object) -> Result<Self, Self::Error> {
        let url = Reflect::get(&value, &JsValue::from_str("url"))
            .ok()
            .and_then(|value| value.as_string())
            .ok_or_else(|| anyhow!("url is not a String"))?
            .parse::<Url>()?;

        let media_type = Reflect::get(&value, &JsValue::from_str("mediaType"))
            .ok()
            .and_then(|value| value.as_string())
            .ok_or_else(|| anyhow!("mediaType is not a String"))?
            .parse::<Mime>()?;

        let file_name = Reflect::get(&value, &JsValue::from_str("fileName"))
            .ok()
            .and_then(|value| value.as_string())
            .ok_or_else(|| anyhow!("fileName is not a String"))?;

        let file_size = Reflect::get(&value, &JsValue::from_str("fileSize"))
            .ok()
            .and_then(|value| {
                if value.is_null() || value.is_undefined() {
                    return None;
                }
                Some(value)
            })
            .map(|value| {
                u64::try_from(BigInt::from(value)).map_err(|_| anyhow!("Could not parse fileSize"))
            })
            .transpose()?;

        Ok(Self {
            url,
            media_type,
            file_name,
            file_size,
        })
    }
}

impl TryFrom<js_sys::Object> for Thumbnail {
    type Error = anyhow::Error;

    fn try_from(value: js_sys::Object) -> Result<Self, Self::Error> {
        let url = Reflect::get(&value, &JsValue::from_str("url"))
            .ok()
            .and_then(|value| value.as_string())
            .ok_or_else(|| anyhow!("url is not a String"))?
            .parse::<Url>()?;

        let media_type = Reflect::get(&value, &JsValue::from_str("mediaType"))
            .ok()
            .and_then(|value| value.as_string())
            .ok_or_else(|| anyhow!("mediaType is not a String"))?
            .parse::<Mime>()?;

        let width = Reflect::get(&value, &JsValue::from_str("width"))
            .ok()
            .and_then(|value| value.as_f64())
            .map(|value| value as u32);

        let height = Reflect::get(&value, &JsValue::from_str("height"))
            .ok()
            .and_then(|value| value.as_f64())
            .map(|value| value as u32);

        Ok(Self {
            url,
            media_type,
            width,
            height,
        })
    }
}
