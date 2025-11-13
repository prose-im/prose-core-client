// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use encryption_domain_service::{EncryptionDomainService, EncryptionDomainServiceDependencies};
pub use noop_encryption_service::NoopEncryptionService;

mod encryption_domain_service;

mod noop_encryption_service;
#[cfg(not(target_arch = "wasm32"))]
pub mod signal_native;
