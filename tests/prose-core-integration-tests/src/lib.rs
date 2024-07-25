// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::Level;

#[cfg(not(target_arch = "wasm32"))]
#[ctor::ctor]
fn init() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        // Set this to Level::DEBUG to log SQL queries…
        .with_max_level(Level::INFO)
        .try_init();
}

#[cfg(test)]
mod tests;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
