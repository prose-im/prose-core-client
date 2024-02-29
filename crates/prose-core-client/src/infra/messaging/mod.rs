// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use caching_message_repository::CachingMessageRepository;
pub use drafts_repository::{DraftsRecord, DraftsRepository};
pub use message_record::MessageRecord;

mod caching_message_repository;
mod drafts_repository;
mod message_archive_service;
mod message_record;
mod messaging_service;
