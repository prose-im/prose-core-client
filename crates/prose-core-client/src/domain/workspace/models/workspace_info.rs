// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::{AvatarId, AvatarMetadata, ServerId};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub name: Option<String>,
    pub icon: Option<WorkspaceIcon>,
    pub accent_color: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct WorkspaceIcon {
    pub id: AvatarId,
    pub owner: ServerId,
    pub mime_type: String,
}

impl WorkspaceIcon {
    pub fn from_metadata(server_id: ServerId, metadata: AvatarMetadata) -> Self {
        Self {
            id: metadata.checksum,
            owner: server_id,
            mime_type: metadata.mime_type,
        }
    }
}
