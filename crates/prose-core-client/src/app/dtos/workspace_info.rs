// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::workspace::models::WorkspaceIcon;

#[derive(Debug, PartialEq, Clone)]
pub struct WorkspaceInfo {
    pub name: String,
    pub icon: Option<WorkspaceIcon>,
    pub accent_color: Option<String>,
}
