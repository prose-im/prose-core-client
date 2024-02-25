// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynConnectedRoomsReadOnlyRepository, DynDraftsRepository, DynRoomFactory,
    DynSidebarDomainService,
};
use crate::domain::rooms::models::{Room, RoomSidebarState};
use crate::domain::shared::models::{RoomId, RoomType};
use crate::dtos::SidebarItem as SidebarItemDTO;

#[derive(InjectDependencies)]
pub struct SidebarService {
    #[inject]
    connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    #[inject]
    drafts_repo: DynDraftsRepository,
    #[inject]
    room_factory: DynRoomFactory,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
}

impl SidebarService {
    pub async fn sidebar_items(&self) -> Vec<SidebarItemDTO> {
        let rooms: Vec<Room> = self.connected_rooms_repo.get_all();
        let mut item_dtos = vec![];

        for room in rooms {
            if room.r#type == RoomType::Unknown || !room.sidebar_state().is_in_sidebar() {
                continue;
            }

            let is_favorite = room.sidebar_state() == RoomSidebarState::Favorite;
            let id = room.room_id.clone();
            let unread_count = room.unread_count();

            let item_dto = SidebarItemDTO {
                name: room.name().unwrap_or_else(|| id.to_string()),
                room: self.room_factory.build(room),
                is_favorite,
                has_draft: self
                    .drafts_repo
                    .get(&id)
                    .await
                    .unwrap_or_default()
                    .is_some(),
                unread_count,
                mentions_count: 0, // TODO
            };
            item_dtos.push(item_dto)
        }

        item_dtos
    }

    pub async fn toggle_favorite(&self, jid: &RoomId) -> Result<()> {
        self.sidebar_domain_service
            .toggle_item_is_favorite(jid)
            .await?;
        Ok(())
    }

    pub async fn remove_from_sidebar(&self, jid: &RoomId) -> Result<()> {
        self.sidebar_domain_service.remove_items(&[jid]).await?;
        Ok(())
    }
}
