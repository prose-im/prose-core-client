// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::BareJid;
use tracing::error;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynBookmarksService, DynClientEventDispatcher, DynConnectedRoomsRepository, DynRoomFactory,
    DynRoomManagementService, DynSidebarRepository,
};
use crate::domain::sidebar::models::{Bookmark, BookmarkType, SidebarItem};
use crate::dtos::SidebarItem as SidebarItemDTO;
use crate::ClientEvent;

#[derive(InjectDependencies)]
pub struct SidebarService {
    #[inject]
    bookmarks_service: DynBookmarksService,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsRepository,
    #[inject]
    room_factory: DynRoomFactory,
    #[inject]
    room_management_service: DynRoomManagementService,
    #[inject]
    sidebar_repo: DynSidebarRepository,
}

impl SidebarService {
    pub fn sidebar_items(&self) -> Vec<SidebarItemDTO> {
        let items = self.sidebar_repo.get_all();
        let mut item_dtos = vec![];

        for item in items {
            let Some(room) = self.connected_rooms_repo.get(&item.jid) else {
                error!(
                    "Couldn't find connected room for sidebar item with jid {}",
                    item.jid
                );
                continue;
            };

            let item_dto = SidebarItemDTO {
                name: item.name,
                room: self.room_factory.build(room),
                is_favorite: item.is_favorite,
                has_draft: false,  // TODO
                unread_count: 0,   // TODO
                mentions_count: 0, // TODO
                error: item.error,
            };
            item_dtos.push(item_dto)
        }

        item_dtos
    }

    pub async fn toggle_favorite(&self, jid: &BareJid) -> Result<()> {
        let Some(mut sidebar_item) = self.sidebar_repo.get(jid) else {
            return Ok(());
        };

        sidebar_item.is_favorite ^= true;

        self.bookmarks_service
            .save_bookmark(&Bookmark::from(&sidebar_item))
            .await?;
        self.sidebar_repo.put(&sidebar_item);
        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }

    pub async fn remove_from_sidebar(&self, jid: &BareJid) -> Result<()> {
        let Some(sidebar_item) = self.sidebar_repo.get(jid) else {
            return Ok(());
        };

        if sidebar_item.r#type == BookmarkType::Group {
            let mut bookmark = Bookmark::from(&sidebar_item);
            bookmark.is_favorite = false;
            bookmark.in_sidebar = false;
            self.bookmarks_service.save_bookmark(&bookmark).await?;
        } else {
            if sidebar_item.r#type == BookmarkType::PrivateChannel {
                let mut bookmark = Bookmark::from(&sidebar_item);
                bookmark.is_favorite = false;
                bookmark.in_sidebar = false;
                self.bookmarks_service.save_bookmark(&bookmark).await?;
            } else {
                self.bookmarks_service.delete_bookmark(&jid).await?;
            }

            if let Some(room) = self.connected_rooms_repo.get(jid) {
                let full_jid = room.info.jid.with_resource_str(&room.info.user_nickname)?;
                self.room_management_service.exit_room(&full_jid).await?;
            }
        }

        self.sidebar_repo.delete(&jid);
        self.client_event_dispatcher
            .dispatch_event(ClientEvent::SidebarChanged);

        Ok(())
    }
}

impl From<&SidebarItem> for Bookmark {
    fn from(value: &SidebarItem) -> Self {
        Self {
            name: value.name.clone(),
            jid: value.jid.clone(),
            r#type: value.r#type.clone(),
            is_favorite: value.is_favorite,
            in_sidebar: true,
        }
    }
}
