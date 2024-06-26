// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_xmpp::mods::AvatarData;

use crate::app::deps::DynUserInfoService;
use crate::domain::shared::models::{AccountId, UserId};
use crate::domain::user_info::models::{AvatarInfo, PlatformImage};
use crate::domain::user_info::repos::AvatarRepository as DomainAvatarRepository;
use crate::infra::avatars::AvatarCache;

pub struct CachingAvatarRepository {
    user_info_service: DynUserInfoService,
    avatar_cache: Box<dyn AvatarCache>,
}

impl CachingAvatarRepository {
    pub fn new(user_info_service: DynUserInfoService, avatar_cache: Box<dyn AvatarCache>) -> Self {
        Self {
            user_info_service,
            avatar_cache,
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl DomainAvatarRepository for CachingAvatarRepository {
    async fn precache_avatar_image(
        &self,
        account: &AccountId,
        user_jid: &UserId,
        info: &AvatarInfo,
    ) -> anyhow::Result<()> {
        if self
            .avatar_cache
            .has_cached_avatar_image(account, user_jid, &info.checksum)
            .await?
        {
            return Ok(());
        }

        let Some(avatar_data) = self
            .user_info_service
            .load_avatar_image(user_jid, &info.checksum)
            .await?
        else {
            return Ok(());
        };

        self.avatar_cache
            .cache_avatar_image(account, user_jid, &avatar_data, info)
            .await?;
        Ok(())
    }

    async fn get(
        &self,
        account: &AccountId,
        user_id: &UserId,
        info: &AvatarInfo,
    ) -> Result<Option<PlatformImage>> {
        self.precache_avatar_image(account, user_id, info).await?;
        let image = self
            .avatar_cache
            .cached_avatar_image(account, user_id, &info.checksum)
            .await?;
        Ok(image)
    }

    async fn set(
        &self,
        account: &AccountId,
        user_jid: &UserId,
        info: &AvatarInfo,
        image: &AvatarData,
    ) -> Result<()> {
        self.avatar_cache
            .cache_avatar_image(account, user_jid, image, info)
            .await?;
        Ok(())
    }

    async fn clear_cache(&self, account: &AccountId) -> Result<()> {
        self.avatar_cache.delete_all_cached_images(account).await?;
        Ok(())
    }
}
