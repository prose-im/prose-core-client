// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;

use crate::domain::shared::models::{AccountId, UserId, UserOrResourceId, UserResourceId};
use crate::domain::user_info::models::{Presence, UserInfo};
use crate::domain::user_info::repos::{UpdateHandler, UserInfoRepository};

use super::PresenceMap;

pub struct InMemoryUserInfoRepository {
    user_infos: RwLock<HashMap<UserId, UserInfo>>,
    presences: RwLock<PresenceMap>,
}

impl InMemoryUserInfoRepository {
    pub fn new() -> Self {
        Self {
            user_infos: Default::default(),
            presences: Default::default(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserInfoRepository for InMemoryUserInfoRepository {
    fn resolve_user_id(&self, _account: &AccountId, user_id: &UserId) -> Option<UserResourceId> {
        let presences = self.presences.read();
        let Some(resource) = presences
            .get_highest_presence(user_id)
            .and_then(|entry| entry.resource.as_deref())
        else {
            return None;
        };

        Some(user_id.with_resource(resource).expect("Invalid resource"))
    }

    async fn set_user_presence(
        &self,
        _account: &AccountId,
        user_id: &UserOrResourceId,
        presence: &Presence,
    ) -> Result<()> {
        let mut map = self.presences.write();
        map.update_presence(user_id, presence.clone().into());
        Ok(())
    }

    async fn get(&self, _account: &AccountId, user_id: &UserId) -> Result<Option<UserInfo>> {
        let mut user_info = self
            .user_infos
            .read()
            .get(user_id)
            .cloned()
            .unwrap_or_default();

        let presence = self
            .presences
            .read()
            .get_highest_presence(user_id)
            .map(|entry| entry.presence.clone())
            .unwrap_or_default();

        user_info.availability = presence.availability;

        Ok(Some(user_info))
    }

    async fn update(
        &self,
        _account: &AccountId,
        user_id: &UserId,
        handler: UpdateHandler,
    ) -> Result<bool> {
        let mut user_infos = self.user_infos.write();
        let user_info = user_infos.entry(user_id.clone()).or_default();
        let user_info_snapshot = user_info.clone();
        handler(user_info);
        Ok(user_info != &user_info_snapshot)
    }

    async fn clear_cache(&self, _account: &AccountId) -> Result<()> {
        self.user_infos.write().clear();
        Ok(())
    }
}
