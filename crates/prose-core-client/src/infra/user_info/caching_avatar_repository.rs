// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::BareJid;

use prose_xmpp::mods::AvatarData;

use crate::app::deps::DynUserInfoService;
use crate::domain::user_info::models::{AvatarInfo, PlatformImage};
use crate::domain::user_info::repos::AvatarRepository as DomainAvatarRepository;
use crate::infra::avatars::AvatarCache;

pub struct CachingAvatarRepository {
    pub(crate) user_info_service: DynUserInfoService,
    pub(crate) avatar_cache: Box<dyn AvatarCache>,
}

#[async_trait]
impl DomainAvatarRepository for CachingAvatarRepository {
    async fn precache_avatar_image(
        &self,
        user_jid: &BareJid,
        info: &AvatarInfo,
    ) -> anyhow::Result<()> {
        if self
            .avatar_cache
            .has_cached_avatar_image(user_jid, &info.checksum)
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
            .cache_avatar_image(user_jid, &avatar_data, info)
            .await?;
        Ok(())
    }

    async fn get(
        &self,
        user_id: &BareJid,
        info: &AvatarInfo,
    ) -> anyhow::Result<Option<PlatformImage>> {
        self.precache_avatar_image(user_id, info).await?;
        let image = self
            .avatar_cache
            .cached_avatar_image(user_id, &info.checksum)
            .await?;
        Ok(image)
    }

    async fn set(
        &self,
        user_jid: &BareJid,
        info: &AvatarInfo,
        image: &AvatarData,
    ) -> anyhow::Result<()> {
        self.avatar_cache
            .cache_avatar_image(user_jid, image, info)
            .await?;
        Ok(())
    }
}
