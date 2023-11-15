// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use bookmarks_service::BookmarksService;
pub use sidebar_domain_service::SidebarDomainService;

mod bookmarks_service;
pub mod impls;
mod sidebar_domain_service;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::bookmarks_service::MockBookmarksService;
    pub use super::sidebar_domain_service::MockSidebarDomainService;
}
