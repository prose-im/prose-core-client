// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::app::deps::{
    DynAppContext, DynAvatarRepository, DynClientEventDispatcher, DynUserInfoService,
    DynWorkspaceInfoRepository,
};
use crate::domain::shared::models::{AvatarInfo, CachePolicy, EntityIdRef};
use crate::domain::user_info::models::PlatformImage;
use crate::domain::workspace::models::{WorkspaceIcon, WorkspaceInfo};
use crate::domain::workspace::services::WorkspaceInfoDomainService as WorkspaceInfoDomainServiceTrait;
use crate::ClientEvent;
use anyhow::Result;
use async_trait::async_trait;
use prose_proc_macros::DependenciesStruct;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(DependenciesStruct)]
pub struct WorkspaceInfoDomainService {
    avatar_repo: DynAvatarRepository,
    client_event_dispatcher: DynClientEventDispatcher,
    ctx: DynAppContext,
    user_info_service: DynUserInfoService,
    workspace_info_repo: DynWorkspaceInfoRepository,

    workspace_icon_loaded: AtomicBool,
    initial_info_change_event_sent: AtomicBool,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl WorkspaceInfoDomainServiceTrait for WorkspaceInfoDomainService {
    async fn get_workspace_info(
        &self,
        _cache_policy: CachePolicy,
    ) -> Result<Option<WorkspaceInfo>> {
        let account = self.ctx.connected_account()?;
        let info = self.workspace_info_repo.get(&account).await?;

        // We're not loading vCard4, because if we have the right to access it, we'll
        // receive it via push, or otherwise it wouldn't make sense to even try to load it.
        Ok(info)
    }

    async fn load_workspace_icon(&self, icon: &WorkspaceIcon) -> Result<Option<PlatformImage>> {
        let account = self.ctx.connected_account()?;
        if let Some(image) = self
            .avatar_repo
            .get(&account, EntityIdRef::from(&icon.owner), &icon.id)
            .await?
        {
            return Ok(Some(image));
        };

        if self.workspace_icon_loaded.swap(true, Ordering::SeqCst) {
            return Ok(None);
        }

        let image_data = self
            .user_info_service
            .load_avatar_image(&icon.owner.to_workspace_entity_id(), &icon.id)
            .await?;

        let Some(image_data) = image_data else {
            return Ok(None);
        };

        self.avatar_repo
            .set(
                &account,
                EntityIdRef::from(&icon.owner),
                &AvatarInfo {
                    checksum: icon.id.clone(),
                    mime_type: icon.mime_type.clone(),
                },
                &image_data,
            )
            .await?;

        self.avatar_repo
            .get(&account, EntityIdRef::from(&icon.owner), &icon.id)
            .await
    }

    async fn handle_workspace_info_changed(&self, info: WorkspaceInfo) -> Result<()> {
        let info_changed = self
            .workspace_info_repo
            .update(
                &self.ctx.connected_account()?,
                Box::new(move |i| {
                    i.name = info.name;
                    i.accent_color = info.accent_color;
                }),
            )
            .await?;

        let initial_change_event_sent = self
            .initial_info_change_event_sent
            .swap(true, Ordering::SeqCst);

        if info_changed || !initial_change_event_sent {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::WorkspaceInfoChanged);
        }

        Ok(())
    }
    async fn handle_icon_changed(&self, icon: Option<WorkspaceIcon>) -> Result<()> {
        let info_changed = self
            .workspace_info_repo
            .update(
                &self.ctx.connected_account()?,
                Box::new(move |i| {
                    i.icon = icon;
                }),
            )
            .await?;

        if info_changed {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::WorkspaceIconChanged);
        }

        Ok(())
    }

    async fn reset_before_reconnect(&self) -> Result<()> {
        self.workspace_icon_loaded.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn clear_cache(&self) -> Result<()> {
        self.workspace_info_repo
            .clear_cache(&self.ctx.connected_account()?)
            .await
    }
}
