// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use block_list_repository::BlockListRepository;
pub use contact_list_repository::ContactListRepository;
pub use presence_sub_requests_repository::PresenceSubRequestsRepository;

mod block_list_repository;
mod contact_list_repository;
mod presence_sub_requests_repository;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::block_list_repository::MockBlockListRepository;
    pub use super::contact_list_repository::MockContactListRepository;
    pub use super::presence_sub_requests_repository::MockPresenceSubRequestsRepository;
}
