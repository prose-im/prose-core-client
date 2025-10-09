// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{AvatarId, ServerId};
use prose_core_client::dtos::{
    WorkspaceIcon as CoreWorkspaceIcon, WorkspaceInfo as CoreWorkspaceInfo,
};

#[derive(uniffi::Record)]
pub struct WorkspaceInfo {
    pub name: String,
    pub icon: Option<WorkspaceIcon>,
    pub accent_color: Option<String>,
}

#[derive(uniffi::Record)]
pub struct WorkspaceIcon {
    pub id: AvatarId,
    pub owner: ServerId,
    pub mime_type: String,
}

impl From<CoreWorkspaceInfo> for WorkspaceInfo {
    fn from(value: CoreWorkspaceInfo) -> Self {
        WorkspaceInfo {
            name: value.name,
            icon: value.icon.map(Into::into),
            accent_color: value.accent_color,
        }
    }
}

impl From<CoreWorkspaceIcon> for WorkspaceIcon {
    fn from(value: CoreWorkspaceIcon) -> Self {
        WorkspaceIcon {
            id: value.id.into(),
            owner: value.owner.into(),
            mime_type: value.mime_type,
        }
    }
}

impl From<WorkspaceIcon> for CoreWorkspaceIcon {
    fn from(value: WorkspaceIcon) -> Self {
        CoreWorkspaceIcon {
            id: value.id.into(),
            owner: value.owner.into(),
            mime_type: value.mime_type,
        }
    }
}
