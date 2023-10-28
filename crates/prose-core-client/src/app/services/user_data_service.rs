// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::{BareJid, Jid};

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAppServiceDependencies, DynAvatarRepository, DynUserInfoRepository,
    DynUserProfileRepository, DynUserProfileService,
};
use crate::domain::user_info::models::{PlatformImage, UserMetadata};
use crate::domain::user_profiles::models::UserProfile;

#[derive(InjectDependencies)]
pub struct UserDataService {
    #[inject]
    app_service: DynAppServiceDependencies,
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
    pub async fn load_avatar(&self, from: &BareJid) -> Result<Option<PlatformImage>> {
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

    pub async fn load_user_profile(&self, from: &BareJid) -> Result<Option<UserProfile>> {
        self.user_profile_repo.get(from).await
    }

    pub async fn load_user_metadata(&self, from: &BareJid) -> Result<Option<UserMetadata>> {
        let Jid::Full(full_jid) = self.user_info_repo.resolve_bare_jid_to_full(from) else {
            return Ok(None);
        };
        let metadata = self
            .user_profile_service
            .load_user_metadata(&full_jid, self.app_service.time_provider.now())
            .await?;
        Ok(metadata)
    }
}
