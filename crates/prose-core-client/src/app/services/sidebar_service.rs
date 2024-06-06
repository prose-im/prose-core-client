// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use tracing::error;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAppContext, DynConnectedRoomsReadOnlyRepository, DynDraftsRepository, DynMessagesRepository,
    DynRoomFactory, DynSidebarDomainService,
};
use crate::domain::rooms::models::{Room, RoomSidebarState};
use crate::domain::shared::models::{RoomId, RoomType};
use crate::dtos::SidebarItem as SidebarItemDTO;

#[derive(InjectDependencies)]
pub struct SidebarService {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    #[inject]
    drafts_repo: DynDraftsRepository,
    #[inject]
    room_factory: DynRoomFactory,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
    #[inject]
    messages_repo: DynMessagesRepository,
}

impl SidebarService {
    pub async fn sidebar_items(&self) -> Vec<SidebarItemDTO> {
        let Ok(account) = self.ctx.connected_account() else {
            error!("Could not read sidebar items since Client is not connected");
            return vec![];
        };

        let rooms: Vec<Room> = self.connected_rooms_repo.get_all(&account);
        let mut item_dtos = vec![];

        for room in rooms {
            if room.r#type == RoomType::Unknown || !room.sidebar_state().is_in_sidebar() {
                continue;
            }

            let stats = room
                .update_statistics_if_needed(&account, &self.messages_repo)
                .await
                .inspect_err(|err| {
                    error!(
                        "Failed to update room statistics for {}. {}",
                        room.room_id,
                        err.to_string()
                    )
                })
                .unwrap_or_default();

            let is_favorite = room.sidebar_state() == RoomSidebarState::Favorite;
            let id = room.room_id.clone();

            let item_dto = SidebarItemDTO {
                name: room.name().unwrap_or_else(|| id.to_string()),
                room: self.room_factory.build(room),
                is_favorite,
                has_draft: self
                    .drafts_repo
                    .get(&account, &id)
                    .await
                    .unwrap_or_default()
                    .is_some(),
                unread_count: stats.unread_count,
                mentions_count: stats.mentions_count,
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
