// prose-core-client/prose-sdk-js
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use wasm_bindgen::JsError;

pub type Result<T, E = JsError> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct WasmError(#[from] anyhow::Error);
