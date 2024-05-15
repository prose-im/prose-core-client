// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use message_archive_domain_service::MessageArchiveDomainService;
pub use message_archive_service::{MessageArchiveService, MessagePage};
pub use message_migration_domain_service::MessageMigrationDomainService;
pub use messaging_service::MessagingService;

pub mod impls;
mod message_archive_domain_service;
mod message_archive_service;
mod message_migration_domain_service;
mod messaging_service;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::message_archive_domain_service::MockMessageArchiveDomainService;
    pub use super::message_archive_service::MockMessageArchiveService;
    pub use super::message_migration_domain_service::MockMessageMigrationDomainService;
    pub use super::messaging_service::MockMessagingService;
}
