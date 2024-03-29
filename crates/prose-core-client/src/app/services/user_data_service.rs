// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAvatarRepository, DynTimeProvider, DynUserInfoRepository, DynUserProfileRepository,
    DynUserProfileService,
};
use crate::domain::shared::models::UserId;
use crate::domain::user_info::models::{PlatformImage, UserMetadata};
use crate::domain::user_profiles::models::UserProfile;

#[derive(InjectDependencies)]
pub struct UserDataService {
    #[inject]
    time_provider: DynTimeProvider,
    #[inject]
    user_profile_service: DynUserProfileService,
    #[inject]
    avatar_repo: DynAvatarRepository,
    #[inject]
    user_info_repo: DynUserInfoRepository,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
}

impl UserDataService {
    pub async fn load_avatar(&self, from: &UserId) -> Result<Option<PlatformImage>> {
        let Some(avatar_metadata) = self
            .user_info_repo
            .get_user_info(from)
            .await?
            .and_then(|info| info.avatar)
        else {
            return Ok(None);
        };
        let image = self.avatar_repo.get(from, &avatar_metadata).await?;
        Ok(image)
    }

    pub async fn load_user_profile(&self, from: &UserId) -> Result<Option<UserProfile>> {
        self.user_profile_repo.get(from).await
    }

    pub async fn load_user_metadata(&self, from: &UserId) -> Result<Option<UserMetadata>> {
        let Some(resource_id) = self
            .user_info_repo
            .resolve_user_id_to_user_resource_id(from)
        else {
            return Ok(None);
        };
        let metadata = self
            .user_profile_service
            .load_user_metadata(&resource_id, self.time_provider.now())
            .await?;
        Ok(metadata)
    }
}
