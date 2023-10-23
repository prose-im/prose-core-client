// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::{BareJid, Jid};

use crate::app::deps::{
    DynAppServiceDependencies, DynAvatarRepository, DynUserInfoRepository,
    DynUserProfileRepository, DynUserProfileService,
};
use crate::domain::user_info::models::{PlatformImage, UserMetadata};
use crate::domain::user_profiles::models::UserProfile;

pub struct UserDataService {
    deps: DynAppServiceDependencies,
    user_profile_service: DynUserProfileService,
    avatar_repository: DynAvatarRepository,
    user_info_repository: DynUserInfoRepository,
    user_profile_repository: DynUserProfileRepository,
}

impl UserDataService {
    pub async fn load_avatar(&self, from: &BareJid) -> Result<Option<PlatformImage>> {
        let Some(avatar_metadata) = self
            .user_info_repository
            .get_user_info(from)
            .await?
            .and_then(|info| info.avatar)
        else {
            return Ok(None);
        };
        let image = self.avatar_repository.get(from, &avatar_metadata).await?;
        Ok(image)
    }

    pub async fn load_user_profile(&self, from: &BareJid) -> Result<Option<UserProfile>> {
        self.user_profile_repository.get(from).await
    }

    pub async fn load_user_metadata(&self, from: &BareJid) -> Result<Option<UserMetadata>> {
        let Jid::Full(full_jid) = self.user_info_repository.resolve_bare_jid_to_full(from) else {
            return Ok(None);
        };
        let metadata = self
            .user_profile_service
            .load_user_metadata(&full_jid, self.deps.time_provider.now())
            .await?;
        Ok(metadata)
    }
}
