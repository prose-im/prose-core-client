// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;

use crate::app::deps::DynBlockListService;
use crate::domain::contacts::repos::BlockListRepository;
use crate::domain::shared::models::AccountId;
use crate::dtos::UserId;

pub struct CachingBlockListRepository {
    service: DynBlockListService,
    blocked_users: RwLock<Option<HashSet<UserId>>>,
}

impl CachingBlockListRepository {
    pub fn new(service: DynBlockListService) -> Self {
        Self {
            service,
            blocked_users: Default::default(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl BlockListRepository for CachingBlockListRepository {
    async fn get_all(&self, _account: &AccountId) -> Result<Vec<UserId>> {
        self.load_block_list_if_needed().await?;
        Ok(self
            .blocked_users
            .read()
            .as_ref()
            .map(|set| set.iter().cloned().collect::<Vec<_>>())
            .unwrap_or_else(|| vec![]))
    }

    async fn insert(&self, _account: &AccountId, user_id: &UserId) -> Result<bool> {
        self.load_block_list_if_needed().await?;

        Ok(self
            .blocked_users
            .write()
            .get_or_insert_with(Default::default)
            .insert(user_id.clone()))
    }

    async fn delete(&self, _account: &AccountId, user_id: &UserId) -> Result<bool> {
        Ok(self
            .blocked_users
            .write()
            .get_or_insert_with(Default::default)
            .remove(user_id))
    }

    async fn delete_all(&self, _account: &AccountId) -> Result<bool> {
        let Some(blocked_users) = &mut *self.blocked_users.write() else {
            return Ok(false);
        };
        let has_entries = !blocked_users.is_empty();
        blocked_users.clear();
        Ok(has_entries)
    }

    async fn reset_before_reconnect(&self, account: &AccountId) -> Result<()> {
        self.clear_cache(account).await
    }

    async fn clear_cache(&self, _account: &AccountId) -> Result<()> {
        self.blocked_users.write().take();
        Ok(())
    }
}

impl CachingBlockListRepository {
    async fn load_block_list_if_needed(&self) -> Result<()> {
        if self.blocked_users.read().is_some() {
            return Ok(());
        }

        let blocked_users = self.service.load_block_list().await?;
        self.blocked_users
            .write()
            .replace(blocked_users.into_iter().collect());
        Ok(())
    }
}
