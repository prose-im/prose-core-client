// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::DynBlockListDomainService;
use crate::app::event_handlers::{
    BlockListEvent, BlockListEventType, ServerEvent, ServerEventHandler,
};

/// Handles block list related events.
#[derive(InjectDependencies)]
pub struct BlockListEventHandler {
    #[inject]
    block_list_domain_service: DynBlockListDomainService,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for BlockListEventHandler {
    fn name(&self) -> &'static str {
        "contact_list"
    }

    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>> {
        match event {
            ServerEvent::BlockList(event) => {
                self.handle_block_list_event(event).await?;
            }
            _ => return Ok(Some(event)),
        }
        Ok(None)
    }
}

impl BlockListEventHandler {
    async fn handle_block_list_event(&self, event: BlockListEvent) -> Result<()> {
        match event.r#type {
            BlockListEventType::UserBlocked { user_id } => {
                self.block_list_domain_service
                    .handle_user_blocked(&user_id)
                    .await?;
            }
            BlockListEventType::UserUnblocked { user_id } => {
                self.block_list_domain_service
                    .handle_user_unblocked(&user_id)
                    .await?;
            }
            BlockListEventType::BlockListCleared => {
                self.block_list_domain_service
                    .handle_block_list_cleared()
                    .await?;
            }
        }

        Ok(())
    }
}
