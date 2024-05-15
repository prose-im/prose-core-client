// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use message_archive_domain_service::{
    MessageArchiveDomainService, MessageArchiveDomainServiceDependencies,
};
pub use message_migration_domain_service::{
    MessageMigrationDomainService, MessageMigrationDomainServiceDependencies,
};

mod message_archive_domain_service;
mod message_migration_domain_service;
