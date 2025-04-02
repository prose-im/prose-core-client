// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use prose_xmpp::{IDProvider, TimeProvider};

use crate::app::deps::app_context::AppContext;
use crate::app::event_handlers::{ClientEventDispatcherTrait, ServerEventHandlerQueue};
use crate::domain::account::services::UserAccountService;
use crate::domain::connection::services::ConnectionService;
use crate::domain::contacts::repos::{
    BlockListRepository, ContactListRepository, PresenceSubRequestsRepository,
};
use crate::domain::contacts::services::{
    BlockListDomainService, BlockListService, ContactListDomainService, ContactListService,
};
use crate::domain::encryption::repos::{
    EncryptionKeysRepository, SessionRepository, UserDeviceRepository,
};
use crate::domain::encryption::services::{
    EncryptionDomainService, EncryptionService, UserDeviceIdProvider, UserDeviceService,
};
use crate::domain::general::services::RequestHandlingService;
use crate::domain::messaging::repos::{
    DraftsRepository, MessagesRepository, OfflineMessagesRepository,
};
use crate::domain::messaging::services::{MessageArchiveDomainService, MessageIdProvider};
use crate::domain::messaging::services::{
    MessageArchiveService, MessageMigrationDomainService, MessagingService,
};
use crate::domain::rooms::repos::{ConnectedRoomsReadOnlyRepository, ConnectedRoomsRepository};
use crate::domain::rooms::services::{
    RoomAttributesService, RoomFactory, RoomManagementService, RoomParticipationService,
    RoomsDomainService,
};
use crate::domain::settings::repos::{AccountSettingsRepository, LocalRoomSettingsRepository};
use crate::domain::settings::services::SyncedRoomSettingsService;
use crate::domain::sidebar::services::{BookmarksService, SidebarDomainService};
use crate::domain::uploads::services::UploadService;
use crate::domain::user_info::repos::{
    AvatarRepository, UserInfoRepository, UserProfileRepository,
};
use crate::domain::user_info::services::{UserInfoDomainService, UserInfoService};
use crate::domain::workspace::repos::WorkspaceInfoRepository;
use crate::domain::workspace::services::WorkspaceInfoDomainService;
use crate::infra::general::RngProvider;

pub type DynAccountSettingsRepository = Arc<dyn AccountSettingsRepository>;
pub type DynAppContext = Arc<AppContext>;
pub type DynAvatarRepository = Arc<dyn AvatarRepository>;
pub type DynBlockListDomainService = Arc<dyn BlockListDomainService>;
pub type DynBlockListRepository = Arc<dyn BlockListRepository>;
pub type DynBlockListService = Arc<dyn BlockListService>;
pub type DynBookmarksService = Arc<dyn BookmarksService>;
pub type DynClientEventDispatcher = Arc<dyn ClientEventDispatcherTrait>;
pub type DynConnectedRoomsReadOnlyRepository = Arc<dyn ConnectedRoomsReadOnlyRepository>;
pub type DynConnectedRoomsRepository = Arc<dyn ConnectedRoomsRepository>;
pub type DynConnectionService = Arc<dyn ConnectionService>;
pub type DynContactListDomainService = Arc<dyn ContactListDomainService>;
pub type DynContactListRepository = Arc<dyn ContactListRepository>;
pub type DynContactListService = Arc<dyn ContactListService>;
pub type DynDraftsRepository = Arc<dyn DraftsRepository>;
pub type DynEncryptionDomainService = Arc<dyn EncryptionDomainService>;
pub type DynEncryptionKeysRepository = Arc<dyn EncryptionKeysRepository>;
pub type DynEncryptionService = Arc<dyn EncryptionService>;
pub type DynIDProvider = Arc<dyn IDProvider>;
pub type DynLocalRoomSettingsRepository = Arc<dyn LocalRoomSettingsRepository>;
pub type DynMessageArchiveDomainService = Arc<dyn MessageArchiveDomainService>;
pub type DynMessageArchiveService = Arc<dyn MessageArchiveService>;
pub type DynMessageIdProvider = Arc<dyn MessageIdProvider>;
pub type DynMessageMigrationDomainService = Arc<dyn MessageMigrationDomainService>;
pub type DynMessagesRepository = Arc<dyn MessagesRepository>;
pub type DynMessagingService = Arc<dyn MessagingService>;
pub type DynOfflineMessagesRepository = Arc<dyn OfflineMessagesRepository>;
pub type DynPresenceSubRequestsRepository = Arc<dyn PresenceSubRequestsRepository>;
pub type DynRequestHandlingService = Arc<dyn RequestHandlingService>;
pub type DynRngProvider = Arc<dyn RngProvider>;
pub type DynRoomAttributesService = Arc<dyn RoomAttributesService>;
pub type DynRoomFactory = RoomFactory;
pub type DynRoomManagementService = Arc<dyn RoomManagementService>;
pub type DynRoomParticipationService = Arc<dyn RoomParticipationService>;
pub type DynRoomsDomainService = Arc<dyn RoomsDomainService>;
pub type DynServerEventHandlerQueue = Arc<ServerEventHandlerQueue>;
pub type DynSessionRepository = Arc<dyn SessionRepository>;
pub type DynSidebarDomainService = Arc<dyn SidebarDomainService>;
pub type DynSyncedRoomSettingsService = Arc<dyn SyncedRoomSettingsService>;
pub type DynTimeProvider = Arc<dyn TimeProvider>;
pub type DynUploadService = Arc<dyn UploadService>;
pub type DynUserAccountService = Arc<dyn UserAccountService>;
pub type DynUserDeviceIdProvider = Arc<dyn UserDeviceIdProvider>;
pub type DynUserDeviceRepository = Arc<dyn UserDeviceRepository>;
pub type DynUserDeviceService = Arc<dyn UserDeviceService>;
pub type DynUserInfoDomainService = Arc<dyn UserInfoDomainService>;
pub type DynUserInfoRepository = Arc<dyn UserInfoRepository>;
pub type DynUserInfoService = Arc<dyn UserInfoService>;
pub type DynUserProfileRepository = Arc<dyn UserProfileRepository>;
pub type DynWorkspaceInfoDomainService = Arc<dyn WorkspaceInfoDomainService>;
pub type DynWorkspaceInfoRepository = Arc<dyn WorkspaceInfoRepository>;

pub struct AppDependencies {
    pub account_settings_repo: DynAccountSettingsRepository,
    pub avatar_repo: DynAvatarRepository,
    pub block_list_domain_service: DynBlockListDomainService,
    pub client_event_dispatcher: DynClientEventDispatcher,
    pub connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    pub connection_service: DynConnectionService,
    pub contact_list_domain_service: DynContactListDomainService,
    pub ctx: DynAppContext,
    pub drafts_repo: DynDraftsRepository,
    pub encryption_domain_service: DynEncryptionDomainService,
    pub id_provider: DynIDProvider,
    pub local_room_settings_repo: DynLocalRoomSettingsRepository,
    pub message_archive_service: DynMessageArchiveService,
    pub message_id_provider: DynMessageIdProvider,
    pub messages_repo: DynMessagesRepository,
    pub messaging_service: DynMessagingService,
    pub offline_messages_repo: DynOfflineMessagesRepository,
    pub request_handling_service: DynRequestHandlingService,
    pub rng_provider: DynRngProvider,
    pub room_attributes_service: DynRoomAttributesService,
    pub room_factory: DynRoomFactory,
    pub room_management_service: DynRoomManagementService,
    pub room_participation_service: DynRoomParticipationService,
    pub rooms_domain_service: DynRoomsDomainService,
    pub server_event_handler_queue: DynServerEventHandlerQueue,
    pub short_id_provider: DynIDProvider,
    pub sidebar_domain_service: DynSidebarDomainService,
    pub time_provider: DynTimeProvider,
    pub upload_service: DynUploadService,
    pub user_account_service: DynUserAccountService,
    pub user_device_repo: DynUserDeviceRepository,
    pub user_info_domain_service: DynUserInfoDomainService,
    pub workspace_info_domain_service: DynWorkspaceInfoDomainService,
    pub workspace_info_repo: DynWorkspaceInfoRepository,
}
