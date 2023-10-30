// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use caching_bookmarks_repository::CachingBookmarksRepository;
pub use in_memory_connected_rooms_repository::InMemoryConnectedRoomsRepository;

mod bookmark_service;
mod caching_bookmarks_repository;
mod in_memory_connected_rooms_repository;
mod room_management_service;
mod room_participation_service;
mod room_topic_service;
