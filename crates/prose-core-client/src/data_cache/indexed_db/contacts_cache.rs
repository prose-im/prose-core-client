use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use indexed_db_futures::prelude::*;
use jid::BareJid;
use microtype::Microtype;
use tracing::debug;

use prose_domain::{Contact, UserProfile};
use prose_xmpp::stanza::avatar::ImageId;
use prose_xmpp::stanza::message::ChatState;

use crate::data_cache::indexed_db::cache::{keys, IndexedDBDataCacheError};
use crate::data_cache::indexed_db::idb_database_ext::{
    IdbDatabaseExt, IdbObjectStoreExtGet, IdbObjectStoreExtSet,
};
use crate::data_cache::indexed_db::IndexedDBDataCache;
use crate::data_cache::ContactsCache;
use crate::domain_ext::Availability;
use crate::types::{roster, AvatarMetadata, Presence};

use super::cache::Result;

#[async_trait(? Send)]
impl ContactsCache for IndexedDBDataCache {
    type Error = IndexedDBDataCacheError;

    async fn set_roster_update_time(&self, timestamp: &DateTime<Utc>) -> Result<()> {
        self.db
            .set_value(
                keys::SETTINGS_STORE,
                keys::settings::ROSTER_UPDATE,
                &timestamp,
            )
            .await
    }

    async fn roster_update_time(&self) -> Result<Option<DateTime<Utc>>> {
        self.db
            .get_value(keys::SETTINGS_STORE, keys::settings::ROSTER_UPDATE)
            .await
    }

    async fn insert_roster_items(&self, items: &[roster::Item]) -> Result<()> {
        debug!("Store {} roster items.", items.len());

        let tx = self.db.transaction_on_one_with_mode(
            keys::ROSTER_ITEMS_STORE,
            IdbTransactionMode::Readwrite,
        )?;
        let store = tx.object_store(keys::ROSTER_ITEMS_STORE)?;

        for item in items {
            store.set_value(item.jid.to_string(), &item)?;
        }

        tx.await.into_result()?;
        Ok(())
    }

    async fn insert_user_profile(&self, jid: &BareJid, profile: &UserProfile) -> Result<()> {
        debug!("Store profile for {}.", jid);
        self.db
            .set_value(keys::USER_PROFILE_STORE, jid.to_string(), &profile)
            .await
    }

    async fn load_user_profile(&self, jid: &BareJid) -> Result<Option<UserProfile>> {
        self.db
            .get_value(keys::USER_PROFILE_STORE, jid.to_string())
            .await
    }

    async fn delete_user_profile(&self, jid: &BareJid) -> Result<()> {
        self.db
            .delete_value(keys::USER_PROFILE_STORE, jid.to_string())
            .await
    }

    async fn insert_avatar_metadata(&self, jid: &BareJid, metadata: &AvatarMetadata) -> Result<()> {
        self.db
            .set_value(keys::AVATAR_METADATA_STORE, jid.to_string(), metadata)
            .await
    }

    async fn load_avatar_metadata(&self, jid: &BareJid) -> Result<Option<AvatarMetadata>> {
        self.db
            .get_value(keys::AVATAR_METADATA_STORE, jid.to_string())
            .await
    }

    async fn insert_presence(&self, jid: &BareJid, presence: &Presence) -> Result<()> {
        self.db
            .set_value(keys::PRESENCE_STORE, jid.to_string(), &presence)
            .await
    }

    async fn insert_chat_state(&self, jid: &BareJid, chat_state: &ChatState) -> Result<()> {
        self.db
            .set_value(keys::CHAT_STATE_STORE, jid.to_string(), &chat_state)
            .await
    }

    async fn load_chat_state(&self, jid: &BareJid) -> Result<Option<ChatState>> {
        self.db
            .get_value(keys::CHAT_STATE_STORE, jid.to_string())
            .await
    }

    async fn load_contacts(&self) -> Result<Vec<(Contact, Option<ImageId>)>> {
        let tx = self.db.transaction_on_multi_with_mode(
            &[
                keys::USER_PROFILE_STORE,
                keys::ROSTER_ITEMS_STORE,
                keys::PRESENCE_STORE,
            ],
            IdbTransactionMode::Readonly,
        )?;

        let roster_items_store = tx.object_store(keys::ROSTER_ITEMS_STORE)?;
        let user_profile_store = tx.object_store(keys::USER_PROFILE_STORE)?;
        let presence_store = tx.object_store(keys::PRESENCE_STORE)?;

        let jids = roster_items_store.get_all_keys()?.await?;
        let mut contacts = vec![];

        for jid in jids {
            let jid_str = jid
                .as_string()
                .ok_or(IndexedDBDataCacheError::InvalidDBKey)?;
            let parsed_jid = BareJid::from_str(&jid_str)?;

            let roster_item = roster_items_store
                .get_value::<roster::Item>(&jid_str)
                .await?;

            let Some(roster_item) = roster_item else {
                continue;
            };

            let user_profile = user_profile_store
                .get_value::<UserProfile>(&jid_str)
                .await?;
            let presence = presence_store.get_value::<Presence>(&jid_str).await?;

            let availability = if let Some(presence) = &presence {
                Availability::from((
                    presence.kind.as_ref().map(|v| v.0.clone()),
                    presence.show.as_ref().map(|v| v.0.clone()),
                ))
                .into_inner()
            } else {
                prose_domain::Availability::Unavailable
            };

            let full_name = user_profile.as_ref().and_then(|u| u.full_name.clone());
            let nickname = user_profile.as_ref().and_then(|u| u.nickname.clone());

            let contact = Contact {
                jid: parsed_jid.clone(),
                name: full_name.or(nickname).unwrap_or(parsed_jid.to_string()),
                avatar: None,
                availability,
                status: presence.and_then(|p| p.status),
                groups: if roster_item.groups.is_empty() {
                    vec!["".to_string()]
                } else {
                    roster_item.groups
                },
            };
            contacts.push((contact, None))
        }

        Ok(contacts)
    }
}
