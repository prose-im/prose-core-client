// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use tracing::error;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAppContext, DynConnectedRoomsReadOnlyRepository, DynDraftsRepository, DynMessagesRepository,
    DynRoomFactory, DynSidebarDomainService, DynUserInfoDomainService, DynUserInfoRepository,
};
use crate::domain::rooms::models::{Participant, Room, RoomSidebarState};
use crate::domain::shared::models::{CachePolicy, RoomId, RoomType};
use crate::domain::user_info::models::UserInfoOptExt;
use crate::dtos::{AvatarBundle, SidebarItem as SidebarItemDTO, SidebarItemType};
use crate::util::textual_palette::{
    generate_textual_initials, generate_textual_palette, normalize_textual_initials,
};

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
    #[inject]
    user_info_domain_service: DynUserInfoDomainService,
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

            let item_type: SidebarItemType = match room.r#type {
                RoomType::DirectMessage => {
                    let participant = room.with_participants(|p| {
                        for part in p.values() {
                            return part.clone();
                        }
                        unreachable!(
                            "Room of type DirectMessage must have at least one participant"
                        );
                    });
                    let user_id = participant
                        .real_id
                        .expect("Participant in DirectMessage must have a user id");

                    let user_info = self
                        .user_info_domain_service
                        .get_user_info(&user_id, CachePolicy::ReturnCacheDataDontLoad)
                        .await
                        .unwrap_or_default()
                        .into_user_presence_info_or_fallback(user_id.clone());

                    SidebarItemType::DirectMessage {
                        user_id,
                        availability: participant.availability,
                        avatar_bundle: user_info.avatar_bundle(),
                        status: user_info.status,
                    }
                }
                RoomType::Group => SidebarItemType::Group,
                RoomType::PrivateChannel => SidebarItemType::PrivateChannel,
                RoomType::PublicChannel => SidebarItemType::PublicChannel,
                RoomType::Generic | RoomType::Unknown => SidebarItemType::Generic,
            };

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
                r#type: item_type,
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
