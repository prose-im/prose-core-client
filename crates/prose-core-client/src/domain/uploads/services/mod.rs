// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use upload_service::UploadService;

mod upload_service;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::upload_service::MockUploadService;
}
