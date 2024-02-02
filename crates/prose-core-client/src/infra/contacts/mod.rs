// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use caching_block_list_repository::CachingBlockListRepository;
pub use caching_contacts_repository::CachingContactsRepository;
pub use presence_sub_requests_repository::PresenceSubRequestsRepository;

mod block_list_service;
mod caching_block_list_repository;
mod caching_contacts_repository;
mod contact_list_service;
mod presence_sub_requests_repository;
