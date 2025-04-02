// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::app::deps::{DynAppContext, DynWorkspaceInfoDomainService};
use crate::domain::shared::models::CachePolicy;
use crate::domain::user_info::models::PlatformImage;
use crate::domain::workspace::models::WorkspaceIcon;
use crate::dtos::WorkspaceInfo as WorkspaceInfoDTO;
use anyhow::Result;
use prose_proc_macros::InjectDependencies;

#[derive(InjectDependencies)]
pub struct WorkspaceService {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    workspace_info_domain_service: DynWorkspaceInfoDomainService,
}

impl WorkspaceService {
    pub async fn load_workspace_info(&self) -> Result<WorkspaceInfoDTO> {
        let info = self
            .workspace_info_domain_service
            .get_workspace_info(CachePolicy::ReturnCacheDataElseLoad)
            .await?;

        let (name, icon, accent_color) = info
            .map(|w| (w.name, w.icon, w.accent_color))
            .unwrap_or_default();

        let name = match name {
            Some(name) => name,
            None => self
                .ctx
                .connected_account()?
                .to_server_id()
                .formatted_name(),
        };

        let info = WorkspaceInfoDTO {
            name,
            icon,
            accent_color,
        };

        Ok(info)
    }

    pub async fn load_workspace_icon(&self, icon: &WorkspaceIcon) -> Result<Option<PlatformImage>> {
        self.workspace_info_domain_service
            .load_workspace_icon(icon)
            .await
    }
}
