// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::anyhow;
use js_sys::Array;
use wasm_bindgen::prelude::*;

use prose_core_client::dtos::UserId;

use crate::client::WasmError;
use crate::types::Message;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Message[]")]
    pub type MessagesArray;

    #[wasm_bindgen(typescript_type = "Reaction[]")]
    pub type ReactionsArray;

    #[wasm_bindgen(typescript_type = "BareJID[]")]
    pub type BareJidArray;

    #[wasm_bindgen(typescript_type = "string[]")]
    pub type StringArray;

    #[wasm_bindgen(typescript_type = "Contact[]")]
    pub type ContactsArray;
}

impl From<Vec<prose_core_client::dtos::Message>> for MessagesArray {
    fn from(value: Vec<prose_core_client::dtos::Message>) -> Self {
        value
            .into_iter()
            .map(|message| Message::from(message))
            .collect_into_js_array::<MessagesArray>()
    }
}

pub trait IntoJSArray {
    fn collect_into_js_array<T: JsCast>(self) -> T;
}

pub trait IntoJSStringArray {
    fn collect_into_js_string_array(self) -> StringArray;
}

impl<I, T> IntoJSStringArray for I
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    fn collect_into_js_string_array(self) -> StringArray {
        self.into_iter()
            .map(|s| JsValue::from_str(s.as_ref()))
            .collect_into_js_array::<StringArray>()
    }
}

impl<I, T> IntoJSArray for I
where
    I: IntoIterator<Item = T>,
    T: Into<JsValue>,
{
    fn collect_into_js_array<U: JsCast>(self) -> U {
        self.into_iter()
            .map(|item| item.into())
            .collect::<js_sys::Array>()
            .unchecked_into::<U>()
    }
}

impl TryFrom<&StringArray> for Vec<String> {
    type Error = WasmError;

    fn try_from(value: &StringArray) -> Result<Self, Self::Error> {
        let js_val: &JsValue = value.as_ref();
        let array: &js_sys::Array = js_val
            .dyn_ref()
            .ok_or_else(|| WasmError::from(anyhow!("The argument must be an array")))?;

        let length: usize = array
            .length()
            .try_into()
            .map_err(|err| WasmError::from(anyhow!("Failed to determine array length. {}", err)))?;

        let mut typed_array = Vec::<String>::with_capacity(length);
        for js in array.iter() {
            let elem = js.as_string().ok_or(WasmError::from(anyhow!(
                "Couldn't unwrap String from Array"
            )))?;
            typed_array.push(elem);
        }

        Ok(typed_array)
    }
}

pub fn try_user_id_vec_from_string_array(arr: Array) -> Result<Vec<UserId>, WasmError> {
    arr.into_iter()
        .map(|value| {
            value
                .as_string()
                .ok_or(anyhow::format_err!(
                    "Could not read String from supposed String Array"
                ))
                .and_then(|str| str.parse::<UserId>().map_err(Into::into))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(WasmError::from)
}
