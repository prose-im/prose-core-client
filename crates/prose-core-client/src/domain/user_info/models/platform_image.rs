// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use base64::engine::general_purpose;
use base64::Engine;

#[cfg(target_arch = "wasm32")]
pub struct PlatformImage {
    pub mime_type: String,
    pub data: Box<[u8]>,
}

#[cfg(target_arch = "wasm32")]
impl PlatformImage {
    pub fn base64(&self) -> String {
        format!(
            "data:{};base64,{}",
            self.mime_type,
            general_purpose::STANDARD.encode(&self.data)
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type PlatformImage = std::path::PathBuf;
