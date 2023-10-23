// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use caching_message_repository::{CachingMessageRepository, MessagesRecord};
pub use drafts_repository::{DraftsRecord, DraftsRepository};

mod caching_message_repository;
mod drafts_repository;
mod message_archive_service;
mod messaging_service;
