// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#![feature(extern_types)]

// Required for wasm-bindgen-derive
extern crate alloc;
extern crate core;

use wasm_bindgen::prelude::*;

mod client;
mod connector;
mod delegate;
mod encryption;
mod error;
mod error_hook;
mod log;
mod types;

#[wasm_bindgen(start)]
pub fn start() {
    error_hook::set_once();
}
