// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use block_list_domain_service::{BlockListDomainService, BlockListDomainServiceDependencies};
pub use contact_list_domain_service::{
    ContactListDomainService, ContactListDomainServiceDependencies,
};

mod block_list_domain_service;
mod contact_list_domain_service;
