use std::fmt::Debug;

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use jid::BareJid;
use microtype::Microtype;
use tracing::{info, instrument};

use prose_domain::{Contact, UserProfile};
use prose_xmpp::mods::{Profile, Roster};
use prose_xmpp::stanza::avatar;
use prose_xmpp::{mods, TimeProvider};

use crate::avatar_cache::AvatarCache;
use crate::data_cache::{ContactsCache, DataCache};
use crate::types::{roster, AvatarMetadata};
use crate::{domain_ext, CachePolicy};

use super::Client;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    pub async fn load_profile(
        &self,
        from: impl Into<BareJid> + Debug,
        cache_policy: CachePolicy,
    ) -> Result<Option<UserProfile>> {
        let from = from.into();

        if cache_policy != CachePolicy::ReloadIgnoringCacheData {
            if let Some(cached_profile) = self.inner.data_cache.load_user_profile(&from).await? {
                info!("Found cached profile for {}", from);
                return Ok(Some(cached_profile));
            }
        }

        if cache_policy == CachePolicy::ReturnCacheDataDontLoad {
            return Ok(None);
        }

        let profile = self.client.get_mod::<mods::Profile>();
        let vcard = profile.load_vcard(from.clone()).await?;

        let Some(vcard) = vcard else { return Ok(None) };

        if vcard.is_empty() {
            return Ok(None);
        }

        let profile = domain_ext::UserProfile::try_from(vcard)?;

        self.inner
            .data_cache
            .insert_user_profile(&from, &profile)
            .await?;
        Ok(Some(profile.into_inner()))
    }

    #[cfg(feature = "native-app")]
    #[instrument]
    pub async fn load_avatar(
        &self,
        from: impl Into<BareJid> + Debug,
        cache_policy: CachePolicy,
    ) -> Result<Option<std::path::PathBuf>> {
        let from = from.into();

        let Some(metadata) = self.load_avatar_metadata(&from, cache_policy).await? else {
            return Ok(None);
        };

        // TODO
        // self.ctx
        //     .load_and_cache_avatar_image(&from, &metadata, cache_policy)
        //     .await

        Ok(None)
    }

    #[instrument]
    pub async fn load_contacts(&self, cache_policy: CachePolicy) -> Result<Vec<Contact>> {
        async fn has_valid_roster_items<D: DataCache, A: AvatarCache>(
            client: &Client<D, A>,
        ) -> Result<bool, <D as ContactsCache>::Error> {
            let Some(last_update) = client.inner.data_cache.roster_update_time().await? else {
                return Ok(false);
            };
            let now: DateTime<Utc> = client.inner.time_provider.now().into();
            Ok(now - last_update <= Duration::minutes(10))
        }

        if cache_policy == CachePolicy::ReloadIgnoringCacheData
            || !has_valid_roster_items(self).await?
        {
            if cache_policy == CachePolicy::ReturnCacheDataDontLoad {
                return Ok(vec![]);
            }

            let roster = self.client.get_mod::<Roster>();
            let roster_items = roster
                .load_roster()
                .await?
                .items
                .into_iter()
                .map(roster::Item::from)
                .collect::<Vec<roster::Item>>();

            self.inner
                .data_cache
                .insert_roster_items(roster_items.as_slice())
                .await
                .ok();

            self.inner
                .data_cache
                .set_roster_update_time(&self.inner.time_provider.now().into())
                .await?;
        }

        let contacts: Vec<(Contact, Option<avatar::ImageId>)> =
            self.inner.data_cache.load_contacts().await?;

        Ok(contacts
            .into_iter()
            .map(|(mut contact, image_id)| {
                if let Some(image_id) = image_id {
                    contact.avatar = self
                        .inner
                        .avatar_cache
                        .cached_avatar_image_url(&contact.jid, &image_id)
                        .and_then(|path| path.into_os_string().into_string().ok())
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
    ) -> Result<Option<AvatarMetadata>> {
        if cache_policy != CachePolicy::ReloadIgnoringCacheData {
            if let Some(metadata) = self.inner.data_cache.load_avatar_metadata(from).await? {
                return Ok(Some(metadata.into()));
            }
        }

        if cache_policy == CachePolicy::ReturnCacheDataDontLoad {
            return Ok(None);
        }

        let profile = self.client.get_mod::<Profile>();
        let metadata = profile
            .load_latest_avatar_metadata(from.clone())
            .await?
            .map(Into::into);

        let Some(metadata) = metadata else {
            return Ok(None);
        };
        self.inner
            .data_cache
            .insert_avatar_metadata(from, &metadata)
            .await?;
        Ok(Some(metadata))
    }
}
