// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use encryption_domain_service::{EncryptionDomainService, EncryptionDomainServiceDependencies};

mod encryption_domain_service;

#[cfg(not(target_arch = "wasm32"))]
pub mod signal_native;
