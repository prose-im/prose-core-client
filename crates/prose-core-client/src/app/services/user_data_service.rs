// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAppContext, DynAvatarRepository, DynEncryptionDomainService, DynUserInfoDomainService,
};
use crate::domain::shared::models::UserId;
use crate::domain::user_info::models::{PlatformImage, UserMetadata, UserProfile};
use crate::dtos::DeviceInfo;

#[derive(InjectDependencies)]
pub struct UserDataService {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    user_info_domain_service: DynUserInfoDomainService,
    #[inject]
    avatar_repo: DynAvatarRepository,
    #[inject]
    encryption_domain_service: DynEncryptionDomainService,
}

impl UserDataService {
    pub async fn load_avatar(&self, user_id: &UserId) -> Result<Option<PlatformImage>> {
        let account = self.ctx.connected_account()?;
        let Some(avatar_metadata) = self
            .user_info_domain_service
            .get_user_info(user_id)
            .await?
            .and_then(|info| info.avatar)
        else {
            return Ok(None);
        };
        let image = self
            .avatar_repo
            .get(&account, user_id, &avatar_metadata)
            .await?;
        Ok(image)
    }

    pub async fn load_user_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>> {
        self.user_info_domain_service
            .get_user_profile(user_id)
            .await
    }

    pub async fn load_user_metadata(&self, user_id: &UserId) -> Result<Option<UserMetadata>> {
        self.user_info_domain_service
            .get_user_metadata(&user_id)
            .await
    }

    pub async fn load_user_device_infos(&self, user_id: &UserId) -> Result<Vec<DeviceInfo>> {
        self.encryption_domain_service
            .load_device_infos(user_id)
            .await
    }
}
