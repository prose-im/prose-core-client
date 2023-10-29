// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use crate::app::deps::app_context::AppContext;
use crate::domain::account::services::UserAccountService;
use crate::domain::connection::services::ConnectionService;
use crate::domain::contacts::repos::ContactsRepository;
use crate::domain::contacts::services::ContactsService;
use crate::domain::general::services::RequestHandlingService;
use crate::domain::messaging::repos::{DraftsRepository, MessagesRepository};
use crate::domain::messaging::services::{MessageArchiveService, MessagingService};
use crate::domain::rooms::repos::{BookmarksRepository, ConnectedRoomsRepository};
use crate::domain::rooms::services::{
    BookmarksService, RoomFactory, RoomManagementService, RoomParticipationService,
    RoomTopicService,
};
use crate::domain::settings::repos::AccountSettingsRepository;
use crate::domain::user_info::repos::{AvatarRepository, UserInfoRepository};
use crate::domain::user_info::services::UserInfoService;
use crate::domain::user_profiles::repos::UserProfileRepository;
use crate::domain::user_profiles::services::UserProfileService;

use super::app_service_dependencies::AppServiceDependencies;

pub(crate) type DynAccountSettingsRepository = Arc<dyn AccountSettingsRepository>;
pub(crate) type DynAppContext = Arc<AppContext>;
pub(crate) type DynAppServiceDependencies = Arc<AppServiceDependencies>;
pub(crate) type DynAvatarRepository = Arc<dyn AvatarRepository>;
pub(crate) type DynBookmarksRepository = Arc<dyn BookmarksRepository>;
pub(crate) type DynBookmarksService = Arc<dyn BookmarksService>;
pub(crate) type DynConnectedRoomsRepository = Arc<dyn ConnectedRoomsRepository>;
pub(crate) type DynConnectionService = Arc<dyn ConnectionService>;
pub(crate) type DynContactsRepository = Arc<dyn ContactsRepository>;
pub(crate) type DynContactsService = Arc<dyn ContactsService>;
pub(crate) type DynDraftsRepository = Arc<dyn DraftsRepository>;
pub(crate) type DynMessageArchiveService = Arc<dyn MessageArchiveService>;
pub(crate) type DynMessagesRepository = Arc<dyn MessagesRepository>;
pub(crate) type DynMessagingService = Arc<dyn MessagingService>;
pub(crate) type DynRequestHandlingService = Arc<dyn RequestHandlingService>;
pub(crate) type DynRoomFactory = RoomFactory;
pub(crate) type DynRoomManagementService = Arc<dyn RoomManagementService>;
pub(crate) type DynRoomParticipationService = Arc<dyn RoomParticipationService>;
pub(crate) type DynRoomTopicService = Arc<dyn RoomTopicService>;
pub(crate) type DynUserAccountService = Arc<dyn UserAccountService>;
pub(crate) type DynUserInfoRepository = Arc<dyn UserInfoRepository>;
pub(crate) type DynUserInfoService = Arc<dyn UserInfoService>;
pub(crate) type DynUserProfileRepository = Arc<dyn UserProfileRepository>;
pub(crate) type DynUserProfileService = Arc<dyn UserProfileService>;

pub struct AppDependencies {
    pub account_settings_repo: DynAccountSettingsRepository,
    pub app_service: DynAppServiceDependencies,
    pub avatar_repo: DynAvatarRepository,
    pub bookmarks_repo: DynBookmarksRepository,
    pub connected_rooms_repo: DynConnectedRoomsRepository,
    pub connection_service: DynConnectionService,
    pub contacts_repo: DynContactsRepository,
    pub contacts_service: DynContactsService,
    pub ctx: DynAppContext,
    pub drafts_repo: DynDraftsRepository,
    pub message_archive_service: DynMessageArchiveService,
    pub messages_repo: DynMessagesRepository,
    pub messaging_service: DynMessagingService,
    pub request_handling_service: DynRequestHandlingService,
    pub room_factory: DynRoomFactory,
    pub room_management_service: DynRoomManagementService,
    pub room_participation_service: DynRoomParticipationService,
    pub room_topic_service: DynRoomTopicService,
    pub user_account_service: DynUserAccountService,
    pub user_info_repo: DynUserInfoRepository,
    pub user_info_service: DynUserInfoService,
    pub user_profile_repo: DynUserProfileRepository,
    pub user_profile_service: DynUserProfileService,
}
