// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::Mutex;

use crate::domain::contacts::models::PresenceSubRequest;
use crate::domain::contacts::repos::PresenceSubRequestsRepository as PresenceSubRequestsRepositoryTrait;
use crate::domain::shared::models::AccountId;
use crate::dtos::UserId;

pub struct PresenceSubRequestsRepository {
    requests: Mutex<Vec<PresenceSubRequest>>,
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
    async fn get_all(&self, _account: &AccountId) -> Result<Vec<PresenceSubRequest>> {
        Ok(self.requests.lock().clone())
    }

    async fn set(&self, _account: &AccountId, request: PresenceSubRequest) -> Result<bool> {
        let mut requests = self.requests.lock();

        if requests
            .iter()
            .find(|req| req.user_id == request.user_id)
            .is_some()
        {
            return Ok(false);
        }

        requests.push(request);
        Ok(true)
    }

    async fn delete(&self, _account: &AccountId, user_id: &UserId) -> Result<bool> {
        let mut requests = self.requests.lock();

        let Some(idx) = requests.iter().position(|req| &req.user_id == user_id) else {
            return Ok(false);
        };

        requests.remove(idx);
        Ok(true)
    }

    async fn clear_cache(&self, _account: &AccountId) -> Result<()> {
        self.requests.lock().clear();
        Ok(())
    }
}
