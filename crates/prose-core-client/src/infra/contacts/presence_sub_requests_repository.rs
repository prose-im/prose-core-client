// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::Mutex;

use crate::domain::contacts::repos::PresenceSubRequestsRepository as PresenceSubRequestsRepositoryTrait;
use crate::domain::shared::models::AccountId;
use crate::dtos::UserId;

pub struct PresenceSubRequestsRepository {
    requests: Mutex<HashSet<UserId>>,
}

impl PresenceSubRequestsRepository {
    pub fn new() -> Self {
        Self {
            requests: Default::default(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl PresenceSubRequestsRepositoryTrait for PresenceSubRequestsRepository {
    async fn get_all(&self, _account: &AccountId) -> Result<Vec<UserId>> {
        Ok(self.requests.lock().iter().cloned().collect::<Vec<_>>())
    }

    async fn set(&self, _account: &AccountId, user_id: &UserId) -> Result<bool> {
        Ok(self.requests.lock().insert(user_id.clone()))
    }

    async fn delete(&self, _account: &AccountId, user_id: &UserId) -> Result<bool> {
        Ok(self.requests.lock().remove(user_id))
    }

    async fn clear_cache(&self, _account: &AccountId) -> Result<()> {
        self.requests.lock().clear();
        Ok(())
    }
}
