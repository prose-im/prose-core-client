// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use workspace_info_repository::{UpdateHandler, WorkspaceInfoRepository};

mod workspace_info_repository;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::workspace_info_repository::MockWorkspaceInfoRepository;
}
