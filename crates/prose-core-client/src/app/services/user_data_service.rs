// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAvatarRepository, DynEncryptionDomainService, DynTimeProvider, DynUserInfoRepository,
    DynUserProfileRepository, DynUserProfileService,
};
use crate::domain::shared::models::UserId;
use crate::domain::user_info::models::{PlatformImage, UserMetadata};
use crate::domain::user_profiles::models::UserProfile;
use crate::dtos::DeviceInfo;

#[derive(InjectDependencies)]
pub struct UserDataService {
    #[inject]
    time_provider: DynTimeProvider,
    #[inject]
    user_profile_service: DynUserProfileService,
    #[inject]
    avatar_repo: DynAvatarRepository,
    #[inject]
    encryption_domain_service: DynEncryptionDomainService,
    #[inject]
    user_info_repo: DynUserInfoRepository,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
}

impl UserDataService {
    pub async fn load_avatar(&self, user_id: &UserId) -> Result<Option<PlatformImage>> {
        let Some(avatar_metadata) = self
            .user_info_repo
            .get_user_info(user_id)
            .await?
            .and_then(|info| info.avatar)
        else {
            return Ok(None);
        };
        let image = self.avatar_repo.get(user_id, &avatar_metadata).await?;
        Ok(image)
    }

    pub async fn load_user_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>> {
        self.user_profile_repo.get(user_id).await
    }

    pub async fn load_user_metadata(&self, user_id: &UserId) -> Result<Option<UserMetadata>> {
        let Some(resource_id) = self
            .user_info_repo
            .resolve_user_id_to_user_resource_id(user_id)
        else {
            return Ok(None);
        };
        let metadata = self
            .user_profile_service
            .load_user_metadata(&resource_id, self.time_provider.now())
            .await?;
        Ok(metadata)
    }

    pub async fn load_user_device_infos(&self, user_id: &UserId) -> Result<Vec<DeviceInfo>> {
        self.encryption_domain_service
            .load_device_infos(user_id)
            .await
    }
}
