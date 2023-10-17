// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jid::BareJid;
use prose_store::prelude::*;
use tracing::debug;

use prose_xmpp::stanza::message::ChatState;

use crate::data_cache::indexed_db::cache::{keys, CacheError};
use crate::data_cache::indexed_db::IndexedDBDataCache;
use crate::data_cache::ContactsCache;
use crate::types::{
    roster, Availability, AvatarMetadata, Contact, Presence, UserActivity, UserProfile,
};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl<D: Driver> ContactsCache for IndexedDBDataCache<D> {
    type Error = CacheError;

    async fn set_roster_update_time(&self, timestamp: &DateTime<Utc>) -> Result<(), Self::Error> {
        self.db
            .transaction_for_reading_and_writing(&[keys::SETTINGS_STORE])
            .await?
            .writeable_collection(keys::SETTINGS_STORE)?
            .put(keys::settings::ROSTER_UPDATE, &timestamp)?;
        Ok(())
    }

    async fn roster_update_time(&self) -> Result<Option<DateTime<Utc>>, Self::Error> {
        let time = self
            .db
            .transaction_for_reading(&[keys::SETTINGS_STORE])
            .await?
            .readable_collection(keys::SETTINGS_STORE)?
            .get(keys::settings::ROSTER_UPDATE)
            .await?;
        Ok(time)
    }

    async fn insert_roster_items(&self, items: &[roster::Item]) -> Result<(), Self::Error> {
        debug!("Store {} roster items.", items.len());

        let tx = self
            .db
            .transaction_for_reading_and_writing(&[keys::ROSTER_ITEMS_STORE])
            .await?;

        {
            let collection = tx.writeable_collection(keys::ROSTER_ITEMS_STORE)?;
            for item in items {
                collection.put(&item.jid.to_string(), &item)?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn insert_user_profile(
        &self,
        jid: &BareJid,
        profile: &UserProfile,
    ) -> Result<(), Self::Error> {
        debug!("Store profile for {}.", jid);
        self.db
            .put(keys::USER_PROFILE_STORE, &jid.to_string(), &profile)
            .await?;
        Ok(())
    }

    async fn load_user_profile(&self, jid: &BareJid) -> Result<Option<UserProfile>, Self::Error> {
        let profile = self
            .db
            .get(keys::USER_PROFILE_STORE, &jid.to_string())
            .await?;
        Ok(profile)
    }

    async fn delete_user_profile(&self, jid: &BareJid) -> Result<(), Self::Error> {
        self.db
            .delete(keys::USER_PROFILE_STORE, &jid.to_string())
            .await?;
        Ok(())
    }

    async fn insert_avatar_metadata(
        &self,
        jid: &BareJid,
        metadata: &AvatarMetadata,
    ) -> Result<(), Self::Error> {
        self.db
            .put(keys::AVATAR_METADATA_STORE, &jid.to_string(), metadata)
            .await?;
        Ok(())
    }

    async fn load_avatar_metadata(
        &self,
        jid: &BareJid,
    ) -> Result<Option<AvatarMetadata>, Self::Error> {
        let metadata = self
            .db
            .get(keys::AVATAR_METADATA_STORE, &jid.to_string())
            .await?;
        Ok(metadata)
    }

    async fn insert_presence(&self, jid: &BareJid, presence: &Presence) -> Result<(), Self::Error> {
        self.db
            .put(keys::PRESENCE_STORE, &jid.to_string(), &presence)
            .await?;
        Ok(())
    }

    async fn insert_user_activity(
        &self,
        jid: &BareJid,
        user_activity: &Option<UserActivity>,
    ) -> Result<(), Self::Error> {
        if let Some(user_activity) = user_activity {
            self.db
                .put(keys::USER_ACTIVITY_STORE, &jid.to_string(), &user_activity)
                .await?;
        } else {
            self.db
                .delete(keys::USER_ACTIVITY_STORE, &jid.to_string())
                .await?;
        }
        Ok(())
    }

    async fn insert_chat_state(
        &self,
        jid: &BareJid,
        chat_state: &ChatState,
    ) -> Result<(), Self::Error> {
        self.db
            .put(keys::CHAT_STATE_STORE, &jid.to_string(), &chat_state)
            .await?;
        Ok(())
    }

    async fn load_chat_state(&self, jid: &BareJid) -> Result<Option<ChatState>, Self::Error> {
        let chat_state = self
            .db
            .get(keys::CHAT_STATE_STORE, &jid.to_string())
            .await?;
        Ok(chat_state)
    }

    async fn load_contacts(&self) -> Result<Vec<Contact>, Self::Error> {
        let tx = self
            .db
            .transaction_for_reading(&[
                keys::USER_PROFILE_STORE,
                keys::ROSTER_ITEMS_STORE,
                keys::PRESENCE_STORE,
                keys::USER_ACTIVITY_STORE,
            ])
            .await?;

        let roster_items = tx.readable_collection(keys::ROSTER_ITEMS_STORE)?;
        let user_profiles = tx.readable_collection(keys::USER_PROFILE_STORE)?;
        let presences = tx.readable_collection(keys::PRESENCE_STORE)?;
        let activities = tx.readable_collection(keys::USER_ACTIVITY_STORE)?;

        let jids = roster_items.all_keys().await?;
        let mut contacts = vec![];

        for jid_str in jids {
            let Some(roster_item) = roster_items.get::<_, roster::Item>(&jid_str).await? else {
                continue;
            };

            let user_profile = user_profiles.get::<_, UserProfile>(&jid_str).await?;
            let presence = presences.get::<_, Presence>(&jid_str).await?;
            let user_activity = activities.get(&jid_str).await?;

            let availability = presence.map(|presence| {
                Availability::from((
                    presence.kind.as_ref().map(|v| v.0.clone()),
                    presence.show.as_ref().map(|v| v.0.clone()),
                ))
            });

            contacts.push(Contact::from((
                roster_item,
                user_profile,
                availability,
                user_activity,
            )))
        }

        Ok(contacts)
    }
}
