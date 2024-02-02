// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use block_list_domain_service::BlockListDomainService;
pub use block_list_service::BlockListService;
pub use contact_list_domain_service::ContactListDomainService;
pub use contact_list_service::ContactListService;

mod block_list_domain_service;
mod block_list_service;
mod contact_list_domain_service;
mod contact_list_service;
pub mod impls;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::block_list_domain_service::MockBlockListDomainService;
    pub use super::block_list_service::MockBlockListService;
    pub use super::contact_list_domain_service::MockContactListDomainService;
    pub use super::contact_list_service::MockContactListService;
}
