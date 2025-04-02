// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::app::deps::DynWorkspaceInfoDomainService;
use crate::app::event_handlers::{
    ServerEvent, ServerEventHandler, WorkspaceInfoEvent, WorkspaceInfoEventType,
};
use crate::domain::workspace::models::WorkspaceIcon;
use anyhow::Result;
use async_trait::async_trait;
use prose_proc_macros::InjectDependencies;

#[derive(InjectDependencies)]
pub struct WorkspaceInfoEventHandler {
    #[inject]
    workspace_info_domain_service: DynWorkspaceInfoDomainService,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for WorkspaceInfoEventHandler {
    fn name(&self) -> &'static str {
        "workspace"
    }

    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>> {
        match event {
            ServerEvent::WorkspaceInfo(event) => {
                self.handle_workspace_info_event(event).await?;
            }
            _ => return Ok(Some(event)),
        }
        Ok(None)
    }
}

impl WorkspaceInfoEventHandler {
    async fn handle_workspace_info_event(&self, event: WorkspaceInfoEvent) -> Result<()> {
        match event.r#type {
            WorkspaceInfoEventType::AvatarChanged { metadata } => {
                let icon = WorkspaceIcon::from_metadata(event.server_id, metadata);
                self.workspace_info_domain_service
                    .handle_icon_changed(Some(icon))
                    .await?;
            }
            WorkspaceInfoEventType::InfoChanged { info } => {
                self.workspace_info_domain_service
                    .handle_workspace_info_changed(info)
                    .await?;
            }
        }

        Ok(())
    }
}
