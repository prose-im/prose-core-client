// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use tracing::error;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynConnectedRoomsReadOnlyRepository, DynRoomFactory, DynSidebarDomainService,
    DynSidebarReadOnlyRepository,
};
use crate::domain::shared::models::RoomId;
use crate::dtos::SidebarItem as SidebarItemDTO;

#[derive(InjectDependencies)]
pub struct SidebarService {
    #[inject]
    connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    #[inject]
    room_factory: DynRoomFactory,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
    #[inject]
    sidebar_repo: DynSidebarReadOnlyRepository,
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
