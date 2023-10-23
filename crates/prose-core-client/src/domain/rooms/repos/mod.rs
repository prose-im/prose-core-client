// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use bookmarks_repository::BookmarksRepository;

mod bookmarks_repository;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::bookmarks_repository::MockBookmarksRepository;
}
