// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use bookmarks_repository::BookmarksRepository;
pub use connected_rooms_repository::{ConnectedRoomsRepository, RoomAlreadyExistsError};

mod bookmarks_repository;
mod connected_rooms_repository;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::bookmarks_repository::MockBookmarksRepository;
    pub use super::connected_rooms_repository::MockConnectedRoomsRepository;
}
