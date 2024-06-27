// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_proc_macros::DependenciesStruct;

use crate::app::deps::{
    DynAppContext, DynAvatarRepository, DynClientEventDispatcher, DynTimeProvider,
    DynUserInfoRepository, DynUserProfileRepository, DynUserProfileService,
};
use crate::domain::shared::models::UserOrResourceId;
use crate::domain::user_info::models::{Avatar, AvatarMetadata, AvatarSource, Presence};
use crate::domain::user_info::services::UserInfoDomainService as UserInfoDomainServiceTrait;
use crate::dtos::{UserId, UserInfo, UserMetadata, UserProfile, UserStatus};
use crate::ClientEvent;

#[derive(DependenciesStruct)]
pub struct UserInfoDomainService {
    pub avatar_repo: DynAvatarRepository,
    pub client_event_dispatcher: DynClientEventDispatcher,
    pub ctx: DynAppContext,
    pub time_provider: DynTimeProvider,
    pub user_info_repo: DynUserInfoRepository,
    pub user_profile_repo: DynUserProfileRepository,
    pub user_profile_service: DynUserProfileService,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserInfoDomainServiceTrait for UserInfoDomainService {
    async fn get_display_name(&self, user_id: &UserId) -> Result<Option<String>> {
        self.user_profile_repo
            .get_display_name(&self.ctx.connected_account()?, user_id)
            .await
    }

    async fn get_user_info(&self, user_id: &UserId) -> Result<Option<UserInfo>> {
        self.user_info_repo
            .get_user_info(&self.ctx.connected_account()?, user_id)
            .await
    }

    async fn get_user_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>> {
        self.user_profile_repo
            .get(&self.ctx.connected_account()?, user_id)
            .await
    }

    async fn get_user_metadata(&self, user_id: &UserId) -> Result<Option<UserMetadata>> {
        let Some(resource_id) = self
            .user_info_repo
            .resolve_user_id(&self.ctx.connected_account()?, user_id)
        else {
            return Ok(None);
        };
        self.user_profile_service
            .load_user_metadata(&resource_id, self.time_provider.now())
            .await
    }

    async fn handle_user_presence_changed(
        &self,
        user_id: &UserOrResourceId,
        presence: &Presence,
    ) -> Result<()> {
        self.user_info_repo
            .set_user_presence(&self.ctx.connected_account()?, user_id, presence)
            .await
    }

    async fn handle_user_status_changed(
        &self,
        user_id: &UserId,
        user_activity: Option<&UserStatus>,
    ) -> Result<()> {
        let account = self.ctx.connected_account()?;
        let is_self_event = account == *user_id;

        self.user_info_repo
            .set_user_activity(&account, &user_id, user_activity)
            .await?;
        self.client_event_dispatcher
            .dispatch_event(ClientEvent::ContactChanged {
                ids: vec![user_id.clone()],
            });

        if is_self_event {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::AccountInfoChanged)
        }

        Ok(())
    }

    async fn handle_avatar_changed(
        &self,
        user_id: &UserId,
        metadata: Option<&AvatarMetadata>,
    ) -> Result<()> {
        let account = self.ctx.connected_account()?;
        let is_self_event = account == *user_id;

        self.user_info_repo
            .set_avatar_metadata(&account, user_id, metadata)
            .await?;

        if let Some(metadata) = metadata {
            self.avatar_repo
                .precache_avatar_image(
                    &account,
                    user_id,
                    &Avatar {
                        id: metadata.checksum.clone(),
                        source: AvatarSource::Pep {
                            mime_type: metadata.mime_type.clone(),
                        },
                        owner: user_id.clone().into(),
                    },
                )
                .await?;
        }

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::AvatarChanged {
                ids: vec![user_id.clone()],
            });

        if is_self_event {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::AccountInfoChanged)
        }

        Ok(())
    }

    async fn handle_user_profile_changed(
        &self,
        user_id: &UserId,
        profile: Option<&UserProfile>,
    ) -> Result<()> {
        let account = self.ctx.connected_account()?;
        let is_self_event = account == *user_id;

        if let Some(profile) = profile {
            self.user_profile_repo
                .set(&account, &user_id, &profile)
                .await?;
        } else {
            self.user_profile_repo.delete(&account, &user_id).await?;
        }

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::ContactChanged {
                ids: vec![user_id.clone()],
            });

        if is_self_event {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::AccountInfoChanged)
        }

        Ok(())
    }

    async fn reset_before_reconnect(&self) -> Result<()> {
        self.user_profile_repo
            .reset_before_reconnect(&self.ctx.connected_account()?)
            .await
    }

    async fn clear_cache(&self) -> Result<()> {
        let account = self.ctx.connected_account()?;
        self.user_info_repo.clear_cache(&account).await?;
        self.user_profile_repo.clear_cache(&account).await?;
        Ok(())
    }
}
