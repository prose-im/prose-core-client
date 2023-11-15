// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use sidebar_repository::{SidebarReadOnlyRepository, SidebarRepository};

mod sidebar_repository;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::sidebar_repository::MockSidebarReadOnlyRepository;
    pub use super::sidebar_repository::MockSidebarReadWriteRepository;
}
