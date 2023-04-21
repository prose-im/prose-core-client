use std::fmt::Debug;
use std::path::PathBuf;

use jid::BareJid;
use microtype::Microtype;
use tracing::{info, instrument};

use prose_core_domain::{Contact, UserProfile};
use prose_core_lib::modules::profile::avatar::ImageId;

use crate::cache::{AvatarCache, DataCache};
use crate::types::AvatarMetadata;
use crate::CachePolicy;

use super::Client;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    pub async fn load_profile(
        &self,
        from: impl Into<BareJid> + Debug,
        cache_policy: CachePolicy,
    ) -> anyhow::Result<UserProfile> {
        let from = from.into();

        if cache_policy != CachePolicy::ReloadIgnoringCacheData {
            if let Some(cached_profile) = self.ctx.data_cache.load_user_profile(&from)? {
                info!("Found cached profile for {}", from);
                return Ok(cached_profile);
            }
        }

        if cache_policy == CachePolicy::ReturnCacheDataDontLoad {
            return Ok(UserProfile::default());
        }

        let Some(profile) = self.ctx.load_vcard(&from).await? else {
          return Ok(UserProfile::default())
        };

        self.ctx.data_cache.insert_user_profile(&from, &profile)?;
        Ok(profile.into_inner())
    }

    #[instrument]
    pub async fn load_avatar(
        &self,
        from: impl Into<BareJid> + Debug,
        cache_policy: CachePolicy,
    ) -> anyhow::Result<Option<PathBuf>> {
        let from = from.into();

        let Some(metadata) = self.load_avatar_metadata(&from, cache_policy).await? else {
          return Ok(None)
        };

        self.ctx
            .load_and_cache_avatar_image(&from, &metadata, cache_policy)
            .await
    }

    #[instrument]
    pub async fn load_contacts(&self, cache_policy: CachePolicy) -> anyhow::Result<Vec<Contact>> {
        if cache_policy == CachePolicy::ReloadIgnoringCacheData
            || !self.ctx.data_cache.has_valid_roster_items()?
        {
            if cache_policy == CachePolicy::ReturnCacheDataDontLoad {
                return Ok(vec![]);
            }

            let roster_items = self.ctx.load_roster().await?;
            self.ctx
                .data_cache
                .insert_roster_items(roster_items.as_slice())
                .ok();
        }

        let contacts: Vec<(Contact, Option<ImageId>)> = self.ctx.data_cache.load_contacts()?;

        Ok(contacts
            .into_iter()
            .map(|(mut contact, image_id)| {
                if let Some(image_id) = image_id {
                    contact.avatar = self
                        .ctx
                        .avatar_cache
                        .cached_avatar_image_url(&contact.jid, &image_id)
                }
                contact
            })
            .collect())
    }
}

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    async fn load_avatar_metadata(
        &self,
        from: &BareJid,
        cache_policy: CachePolicy,
    ) -> anyhow::Result<Option<AvatarMetadata>> {
        if cache_policy != CachePolicy::ReloadIgnoringCacheData {
            if let Some(metadata) = self.ctx.data_cache.load_avatar_metadata(from)? {
                return Ok(Some(metadata));
            }
        }

        if cache_policy == CachePolicy::ReturnCacheDataDontLoad {
            return Ok(None);
        }

        let Some(metadata) = self.ctx.load_latest_avatar_metadata(from).await? else {
      return Ok(None)
    };
        self.ctx
            .data_cache
            .insert_avatar_metadata(from, &metadata)?;
        Ok(Some(metadata))
    }
}
