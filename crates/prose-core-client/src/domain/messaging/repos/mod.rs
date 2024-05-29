// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use drafts_repository::DraftsRepository;
pub use messages_repository::MessagesRepository;
pub use offline_messages_repository::OfflineMessagesRepository;

mod drafts_repository;
mod messages_repository;
mod offline_messages_repository;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::drafts_repository::MockDraftsRepository;
    pub use super::messages_repository::MockMessagesRepository;
    pub use super::offline_messages_repository::MockOfflineMessagesRepository;
}
