// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use message_archive_service::MessageArchiveService;
pub use messaging_service::MessagingService;

mod message_archive_service;
mod messaging_service;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::message_archive_service::MockMessageArchiveService;
    pub use super::messaging_service::MockMessagingService;
}
