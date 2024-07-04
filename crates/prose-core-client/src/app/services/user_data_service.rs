// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{DynEncryptionDomainService, DynUserInfoDomainService};
use crate::domain::shared::models::{CachePolicy, UserId};
use crate::domain::user_info::models::PlatformImage;
use crate::dtos::{Avatar, DeviceInfo, UserMetadata, UserProfile};

#[derive(InjectDependencies)]
pub struct UserDataService {
    #[inject]
    user_info_domain_service: DynUserInfoDomainService,
    #[inject]
    encryption_domain_service: DynEncryptionDomainService,
}

impl UserDataService {
    pub async fn load_avatar(&self, avatar: &Avatar) -> Result<Option<PlatformImage>> {
        self.user_info_domain_service
            .load_avatar_image(avatar)
            .await
    }

    pub async fn load_user_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>> {
        Ok(self
            .user_info_domain_service
            .get_user_profile(user_id, CachePolicy::ReturnCacheDataElseLoad)
            .await?
            .map(Into::into))
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
