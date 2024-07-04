// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(not(target_arch = "wasm32"))]
#[ctor::ctor]
fn init() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
}

#[cfg(test)]
mod tests;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
