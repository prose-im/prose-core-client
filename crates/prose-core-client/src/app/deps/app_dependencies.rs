// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use prose_xmpp::{IDProvider, TimeProvider};

use crate::app::deps::app_context::AppContext;
use crate::app::event_handlers::ClientEventDispatcherTrait;
use crate::domain::account::services::UserAccountService;
use crate::domain::connection::services::ConnectionService;
use crate::domain::contacts::repos::ContactsRepository;
use crate::domain::contacts::services::ContactsService;
use crate::domain::general::services::RequestHandlingService;
use crate::domain::messaging::repos::{DraftsRepository, MessagesRepository};
use crate::domain::messaging::services::{
    MessageArchiveService, MessageMigrationDomainService, MessagingService,
};
use crate::domain::rooms::repos::{ConnectedRoomsReadOnlyRepository, ConnectedRoomsRepository};
use crate::domain::rooms::services::{
    RoomAttributesService, RoomFactory, RoomManagementService, RoomParticipationService,
    RoomsDomainService,
};
use crate::domain::settings::repos::AccountSettingsRepository;
use crate::domain::sidebar::repos::{SidebarReadOnlyRepository, SidebarRepository};
use crate::domain::sidebar::services::{BookmarksService, SidebarDomainService};
use crate::domain::user_info::repos::{AvatarRepository, UserInfoRepository};
use crate::domain::user_info::services::UserInfoService;
use crate::domain::user_profiles::repos::UserProfileRepository;
use crate::domain::user_profiles::services::UserProfileService;

pub type DynAccountSettingsRepository = Arc<dyn AccountSettingsRepository>;
pub type DynAppContext = Arc<AppContext>;
pub type DynAvatarRepository = Arc<dyn AvatarRepository>;
pub type DynBookmarksService = Arc<dyn BookmarksService>;
pub type DynClientEventDispatcher = Arc<dyn ClientEventDispatcherTrait>;
pub type DynConnectedRoomsReadOnlyRepository = Arc<dyn ConnectedRoomsReadOnlyRepository>;
pub type DynConnectedRoomsRepository = Arc<dyn ConnectedRoomsRepository>;
pub type DynConnectionService = Arc<dyn ConnectionService>;
pub type DynContactsRepository = Arc<dyn ContactsRepository>;
pub type DynContactsService = Arc<dyn ContactsService>;
pub type DynDraftsRepository = Arc<dyn DraftsRepository>;
pub type DynIDProvider = Arc<dyn IDProvider>;
pub type DynMessageArchiveService = Arc<dyn MessageArchiveService>;
pub type DynMessagesRepository = Arc<dyn MessagesRepository>;
pub type DynMessagingService = Arc<dyn MessagingService>;
pub type DynRequestHandlingService = Arc<dyn RequestHandlingService>;
pub type DynRoomAttributesService = Arc<dyn RoomAttributesService>;
pub type DynRoomFactory = RoomFactory;
pub type DynRoomManagementService = Arc<dyn RoomManagementService>;
pub type DynRoomParticipationService = Arc<dyn RoomParticipationService>;
pub type DynRoomsDomainService = Arc<dyn RoomsDomainService>;
pub type DynSidebarDomainService = Arc<dyn SidebarDomainService>;
pub type DynSidebarReadOnlyRepository = Arc<dyn SidebarReadOnlyRepository>;
pub type DynSidebarRepository = Arc<dyn SidebarRepository>;
pub type DynTimeProvider = Arc<dyn TimeProvider>;
pub type DynUserAccountService = Arc<dyn UserAccountService>;
pub type DynUserInfoRepository = Arc<dyn UserInfoRepository>;
pub type DynUserInfoService = Arc<dyn UserInfoService>;
pub type DynUserProfileRepository = Arc<dyn UserProfileRepository>;
pub type DynUserProfileService = Arc<dyn UserProfileService>;
pub type DynMessageMigrationDomainService = Arc<dyn MessageMigrationDomainService>;

pub struct AppDependencies {
    pub account_settings_repo: DynAccountSettingsRepository,
    pub avatar_repo: DynAvatarRepository,
    pub client_event_dispatcher: DynClientEventDispatcher,
    pub connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    pub connection_service: DynConnectionService,
    pub contacts_repo: DynContactsRepository,
    pub contacts_service: DynContactsService,
    pub ctx: DynAppContext,
    pub drafts_repo: DynDraftsRepository,
    pub id_provider: DynIDProvider,
    pub message_archive_service: DynMessageArchiveService,
    pub messages_repo: DynMessagesRepository,
    pub messaging_service: DynMessagingService,
    pub request_handling_service: DynRequestHandlingService,
    pub room_attributes_service: DynRoomAttributesService,
    pub room_factory: DynRoomFactory,
    pub room_management_service: DynRoomManagementService,
    pub room_participation_service: DynRoomParticipationService,
    pub rooms_domain_service: DynRoomsDomainService,
    pub short_id_provider: DynIDProvider,
    pub sidebar_domain_service: DynSidebarDomainService,
    pub sidebar_repo: DynSidebarReadOnlyRepository,
    pub time_provider: DynTimeProvider,
    pub user_account_service: DynUserAccountService,
    pub user_info_repo: DynUserInfoRepository,
    pub user_info_service: DynUserInfoService,
    pub user_profile_repo: DynUserProfileRepository,
    pub user_profile_service: DynUserProfileService,
}
