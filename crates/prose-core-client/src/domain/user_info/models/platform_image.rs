// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(target_arch = "wasm32")]
pub type PlatformImage = String;

#[cfg(not(target_arch = "wasm32"))]
pub type PlatformImage = std::path::PathBuf;
