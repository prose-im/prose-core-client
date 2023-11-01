// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use bookmarks_service::BookmarksService;
pub use room_factory::RoomFactory;
pub use room_management_service::RoomManagementService;
pub use room_participation_service::RoomParticipationService;
pub use room_topic_service::RoomTopicService;
pub use rooms_domain_service::{
    CreateOrEnterRoomRequest, CreateOrEnterRoomRequestType, CreateRoomType, RoomsDomainService,
};

mod bookmarks_service;
pub mod impls;
mod room_factory;
mod room_management_service;
mod room_participation_service;
mod room_topic_service;
mod rooms_domain_service;

#[cfg(feature = "test")]
pub mod mocks {
    pub use super::bookmarks_service::MockBookmarksService;
    pub use super::room_management_service::MockRoomManagementService;
    pub use super::room_participation_service::MockRoomParticipationService;
    pub use super::room_topic_service::MockRoomTopicService;
    pub use super::rooms_domain_service::MockRoomsDomainService;
}
