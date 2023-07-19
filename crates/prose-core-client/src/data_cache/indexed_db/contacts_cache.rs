use std::str::FromStr;

use async_trait::async_trait;
use gloo_utils::format::JsValueSerdeExt;
use indexed_db_futures::prelude::*;
use jid::BareJid;
use tracing::debug;
use wasm_bindgen::JsValue;
use xmpp_parsers::presence;

use prose_domain::{Availability, Contact, UserProfile};
use prose_xmpp::stanza::avatar::ImageId;
use prose_xmpp::stanza::message::ChatState;

use crate::data_cache::indexed_db::cache::{keys, IndexedDBDataCacheError};
use crate::data_cache::indexed_db::IndexedDBDataCache;
use crate::data_cache::ContactsCache;
use crate::types::{roster, AvatarMetadata};

use super::cache::Result;

#[async_trait(? Send)]
impl ContactsCache for IndexedDBDataCache {
    type Error = IndexedDBDataCacheError;

    async fn has_valid_roster_items(&self) -> Result<bool> {
        Ok(false)
    }

    async fn insert_roster_items(&self, items: &[roster::Item]) -> Result<()> {
        debug!("Store {} roster items.", items.len());

        let tx = self.db.transaction_on_one_with_mode(
            keys::ROSTER_ITEMS_STORE,
            IdbTransactionMode::Readwrite,
        )?;
        let store = tx.object_store(keys::ROSTER_ITEMS_STORE)?;

        for item in items {
            store.put_key_val(
                &JsValue::from_str(&item.jid.to_string()),
                &JsValue::from_serde(item)?,
            )?;
        }

        tx.await.into_result()?;
        Ok(())
    }

    async fn insert_user_profile(&self, jid: &BareJid, profile: &UserProfile) -> Result<()> {
        debug!("Store profile for {}.", jid);

        let tx = self.db.transaction_on_one_with_mode(
            keys::USER_PROFILE_STORE,
            IdbTransactionMode::Readwrite,
        )?;
        let store = tx.object_store(keys::USER_PROFILE_STORE)?;

        store.put_key_val(
            &JsValue::from_str(&jid.to_string()),
            &JsValue::from_serde(profile)?,
        )?;

        tx.await.into_result()?;

        Ok(())
    }

    async fn load_user_profile(&self, _jid: &BareJid) -> Result<Option<UserProfile>> {
        Ok(None)
    }

    async fn delete_user_profile(&self, _jid: &BareJid) -> Result<()> {
        Ok(())
    }

    async fn insert_avatar_metadata(
        &self,
        _jid: &BareJid,
        _metadata: &AvatarMetadata,
    ) -> Result<()> {
        Ok(())
    }

    async fn load_avatar_metadata(&self, _jid: &BareJid) -> Result<Option<AvatarMetadata>> {
        Ok(None)
    }

    async fn insert_presence(
        &self,
        _jid: &BareJid,
        _kind: Option<presence::Type>,
        _show: Option<presence::Show>,
        _status: Option<String>,
    ) -> Result<()> {
        Ok(())
    }

    async fn insert_chat_state(&self, _jid: &BareJid, _chat_state: &ChatState) -> Result<()> {
        Ok(())
    }

    async fn load_chat_state(&self, _jid: &BareJid) -> Result<Option<ChatState>> {
        Ok(None)
    }

    async fn load_contacts(&self) -> Result<Vec<(Contact, Option<ImageId>)>> {
        let tx = self.db.transaction_on_multi_with_mode(
            &[keys::USER_PROFILE_STORE, keys::ROSTER_ITEMS_STORE],
            IdbTransactionMode::Readonly,
        )?;

        let roster_items_store = tx.object_store(keys::ROSTER_ITEMS_STORE)?;
        let user_profile_store = tx.object_store(keys::USER_PROFILE_STORE)?;
        let jids = roster_items_store.get_all_keys()?.await?;
        let mut contacts = vec![];

        for jid in jids {
            let parsed_jid = BareJid::from_str(
                &jid.as_string()
                    .ok_or(IndexedDBDataCacheError::InvalidDBKey)?,
            )?;

            let Some(roster_item): Option<roster::Item> = roster_items_store
                .get(&jid)?
                .await?
                .map(|v| v.into_serde())
                .transpose()?
            else {
                continue;
            };

            let user_profile: Option<UserProfile> = user_profile_store
                .get(&jid)?
                .await?
                .map(|v| v.into_serde())
                .transpose()?;

            let full_name = user_profile.as_ref().and_then(|u| u.full_name.clone());
            let nickname = user_profile.as_ref().and_then(|u| u.nickname.clone());

            let contact = Contact {
                jid: parsed_jid.clone(),
                name: full_name.or(nickname).unwrap_or(parsed_jid.to_string()),
                avatar: None,
                availability: Availability::Unavailable,
                status: None,
                groups: roster_item.groups,
            };
            contacts.push((contact, None))
        }

        Ok(contacts)
    }
}
