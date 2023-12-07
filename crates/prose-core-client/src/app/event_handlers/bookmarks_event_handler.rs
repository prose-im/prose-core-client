// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::DynSidebarDomainService;
use crate::app::event_handlers::{ServerEvent, ServerEventHandler, SidebarBookmarkEvent};

#[derive(InjectDependencies)]
pub struct BookmarksEventHandler {
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for BookmarksEventHandler {
    fn name(&self) -> &'static str {
        "bookmarks"
    }

    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>> {
        match event {
            ServerEvent::SidebarBookmark(event) => self.handle_bookmark_event(event).await?,
            _ => return Ok(Some(event)),
        }
        Ok(None)
    }
}

impl BookmarksEventHandler {
    async fn handle_bookmark_event(&self, event: SidebarBookmarkEvent) -> Result<()> {
        match event {
            SidebarBookmarkEvent::AddedOrUpdated { bookmarks } => {
                self.sidebar_domain_service
                    .extend_items_from_bookmarks(bookmarks)
                    .await?;
            }
            SidebarBookmarkEvent::Deleted { ids } => {
                self.sidebar_domain_service
                    .handle_removed_items(ids.as_slice())
                    .await?;
            }
            SidebarBookmarkEvent::Purged => {
                self.sidebar_domain_service.handle_remote_purge().await?;
            }
        }

        Ok(())
    }
}
