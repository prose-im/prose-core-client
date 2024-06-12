// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_proc_macros::DependenciesStruct;

use crate::app::deps::{
    DynAppContext, DynBlockListRepository, DynBlockListService, DynClientEventDispatcher,
};
use crate::dtos::UserId;
use crate::ClientEvent;

use super::super::BlockListDomainService as BlockListDomainServiceTrait;

#[derive(DependenciesStruct)]
pub struct BlockListDomainService {
    ctx: DynAppContext,
    block_list_repo: DynBlockListRepository,
    block_list_service: DynBlockListService,
    client_event_dispatcher: DynClientEventDispatcher,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl BlockListDomainServiceTrait for BlockListDomainService {
    async fn load_block_list(&self) -> Result<Vec<UserId>> {
        self.block_list_repo
            .get_all(&self.ctx.connected_account()?)
            .await
    }

    async fn block_user(&self, user_id: &UserId) -> Result<()> {
        self.block_list_service.block_user(user_id).await?;

        if self
            .block_list_repo
            .insert(&self.ctx.connected_account()?, user_id)
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::BlockListChanged);
        }

        Ok(())
    }

    async fn unblock_user(&self, user_id: &UserId) -> Result<()> {
        self.block_list_service.unblock_user(user_id).await?;

        if self
            .block_list_repo
            .delete(&self.ctx.connected_account()?, user_id)
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::BlockListChanged);
        }

        Ok(())
    }

    async fn clear_block_list(&self) -> Result<()> {
        self.block_list_service.clear_block_list().await?;

        if self
            .block_list_repo
            .delete_all(&self.ctx.connected_account()?)
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::BlockListChanged);
        }

        Ok(())
    }

    async fn handle_user_blocked(&self, user_id: &UserId) -> Result<()> {
        if self
            .block_list_repo
            .insert(&self.ctx.connected_account()?, user_id)
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::BlockListChanged);
        }
        Ok(())
    }

    async fn handle_user_unblocked(&self, user_id: &UserId) -> Result<()> {
        if self
            .block_list_repo
            .delete(&self.ctx.connected_account()?, user_id)
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::BlockListChanged);
        }
        Ok(())
    }

    async fn handle_block_list_cleared(&self) -> Result<()> {
        if self
            .block_list_repo
            .delete_all(&self.ctx.connected_account()?)
            .await?
        {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::BlockListChanged);
        }
        Ok(())
    }

    async fn reset_before_reconnect(&self) -> Result<()> {
        self.block_list_repo
            .reset_before_reconnect(&self.ctx.connected_account()?)
            .await?;
        Ok(())
    }

    async fn clear_cache(&self) -> Result<()> {
        self.block_list_repo
            .clear_cache(&self.ctx.connected_account()?)
            .await?;
        Ok(())
    }
}
