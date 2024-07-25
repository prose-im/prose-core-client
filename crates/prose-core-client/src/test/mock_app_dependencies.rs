// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use derivative::Derivative;
use jid::BareJid;
use parking_lot::RwLock;

use prose_xmpp::bare;
use prose_xmpp::test::IncrementingIDProvider;

use crate::app::deps::{
    AppContext, AppDependencies, DynAppContext, DynBookmarksService, DynClientEventDispatcher,
    DynDraftsRepository, DynEncryptionDomainService, DynIDProvider, DynMessageArchiveService,
    DynMessageIdProvider, DynMessagesRepository, DynMessagingService, DynRngProvider,
    DynRoomAttributesService, DynRoomParticipationService, DynSidebarDomainService,
    DynSyncedRoomSettingsService, DynTimeProvider, DynUserInfoDomainService,
};
use crate::app::event_handlers::{MockClientEventDispatcherTrait, ServerEventHandlerQueue};
use crate::app::services::RoomInner;
use crate::domain::account::services::mocks::MockUserAccountService;
use crate::domain::connection::models::{ConnectionProperties, ServerFeatures};
use crate::domain::connection::services::mocks::MockConnectionService;
use crate::domain::contacts::repos::mocks::MockBlockListRepository;
use crate::domain::contacts::services::mocks::{
    MockBlockListDomainService, MockContactListDomainService,
};
use crate::domain::encryption::repos::mocks::MockUserDeviceRepository;
use crate::domain::encryption::services::mocks::MockEncryptionDomainService;
use crate::domain::general::models::Capabilities;
use crate::domain::general::services::mocks::MockRequestHandlingService;
use crate::domain::messaging::repos::mocks::{
    MockDraftsRepository, MockMessagesRepository, MockOfflineMessagesRepository,
};
use crate::domain::messaging::services::mocks::{
    MockMessageArchiveDomainService, MockMessageArchiveService, MockMessageMigrationDomainService,
    MockMessagingService,
};
use crate::domain::messaging::services::WrappingMessageIdProvider;
use crate::domain::rooms::repos::mocks::{
    MockConnectedRoomsReadOnlyRepository, MockConnectedRoomsReadWriteRepository,
};
use crate::domain::rooms::services::impls::RoomsDomainServiceDependencies;
use crate::domain::rooms::services::mocks::{
    MockRoomAttributesService, MockRoomManagementService, MockRoomParticipationService,
    MockRoomsDomainService,
};
use crate::domain::rooms::services::RoomFactory;
use crate::domain::settings::repos::mocks::{
    MockAccountSettingsRepository, MockLocalRoomSettingsRepository,
};
use crate::domain::settings::services::mocks::MockSyncedRoomSettingsService;
use crate::domain::shared::models::{AccountId, ConnectionState};
use crate::domain::sidebar::services::impls::SidebarDomainServiceDependencies;
use crate::domain::sidebar::services::mocks::{MockBookmarksService, MockSidebarDomainService};
use crate::domain::uploads::services::mocks::MockUploadService;
use crate::domain::user_info::repos::mocks::{
    MockAvatarRepository, MockUserInfoRepository, MockUserProfileRepository,
};
use crate::domain::user_info::services::impls::UserInfoDomainServiceDependencies;
use crate::domain::user_info::services::mocks::{MockUserInfoDomainService, MockUserInfoService};
use crate::dtos::{DecryptionContext, UserResourceId};
use crate::infra::general::mocks::StepRngProvider;
use crate::infra::general::OsRngProvider;
use crate::test::ConstantTimeProvider;
use crate::user_resource_id;

pub fn mock_reference_date() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 09, 06, 0, 0, 0).unwrap().into()
}

pub fn mock_muc_service() -> BareJid {
    bare!("conference.prose.org")
}

pub fn mock_account_jid() -> UserResourceId {
    user_resource_id!("jane.doe@prose.org/macOS")
}

pub fn mock_account() -> AccountId {
    bare!("jane.doe@prose.org").into()
}

impl Default for AppContext {
    fn default() -> Self {
        AppContext {
            connection_properties: RwLock::new(Some(ConnectionProperties {
                connection_timestamp: Default::default(),
                connected_jid: mock_account_jid(),
                server_features: ServerFeatures {
                    muc_service: Some(mock_muc_service()),
                    http_upload_service: None,
                    mam_version: None,
                    vcard4: false,
                    avatar_pep_vcard_conversion: false,
                    server_time_offset: Default::default(),
                },
                rooms_caught_up: false,
                decryption_context: Some(DecryptionContext::default()),
            })),
            connection_state: RwLock::new(ConnectionState::Connected),
            capabilities: Capabilities::new("Prose", "https://prose.org", vec![]),
            software_version: Default::default(),
            config: Default::default(),
        }
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct MockAppDependencies {
    pub account_settings_repo: MockAccountSettingsRepository,
    pub avatar_repo: MockAvatarRepository,
    pub block_list_domain_service: MockBlockListDomainService,
    pub bookmarks_service: MockBookmarksService,
    pub client_event_dispatcher: MockClientEventDispatcherTrait,
    pub connected_rooms_repo: MockConnectedRoomsReadOnlyRepository,
    pub connection_service: MockConnectionService,
    pub contact_list_domain_service: MockContactListDomainService,
    pub ctx: AppContext,
    pub drafts_repo: MockDraftsRepository,
    pub encryption_domain_service: MockEncryptionDomainService,
    #[derivative(Default(value = "Arc::new(IncrementingIDProvider::new(\"id\"))"))]
    pub id_provider: DynIDProvider,
    pub local_room_settings_repo: MockLocalRoomSettingsRepository,
    pub message_archive_service: MockMessageArchiveService,
    #[derivative(Default(
        value = "Arc::new(WrappingMessageIdProvider::incrementing(\"msg-id\"))"
    ))]
    pub message_id_provider: DynMessageIdProvider,
    pub messages_repo: MockMessagesRepository,
    pub messaging_service: MockMessagingService,
    pub offline_message_repo: MockOfflineMessagesRepository,
    pub synced_room_settings_service: MockSyncedRoomSettingsService,
    pub request_handling_service: MockRequestHandlingService,
    #[derivative(Default(value = "Arc::new(StepRngProvider::default())"))]
    pub rng_provider: DynRngProvider,
    pub rooms_domain_service: MockRoomsDomainService,
    pub room_management_service: MockRoomManagementService,
    pub room_participation_service: MockRoomParticipationService,
    pub room_attributes_service: MockRoomAttributesService,
    #[derivative(Default(value = "Arc::new(IncrementingIDProvider::new(\"short-id\"))"))]
    pub short_id_provider: DynIDProvider,
    pub sidebar_domain_service: MockSidebarDomainService,
    #[derivative(Default(value = "Arc::new(ConstantTimeProvider::new(mock_reference_date()))"))]
    pub time_provider: DynTimeProvider,
    pub upload_service: MockUploadService,
    pub user_account_service: MockUserAccountService,
    pub user_device_repo: MockUserDeviceRepository,
    pub user_info_domain_service: MockUserInfoDomainService,
}

impl MockAppDependencies {
    pub fn into_deps(self) -> AppDependencies {
        AppDependencies::from(self)
    }
}

impl From<MockAppDependencies> for AppDependencies {
    fn from(mock: MockAppDependencies) -> Self {
        let client_event_dispatcher = Arc::new(mock.client_event_dispatcher);
        let connected_rooms_repo = Arc::new(mock.connected_rooms_repo);
        let ctx = Arc::new(mock.ctx);
        let drafts_repo = Arc::new(mock.drafts_repo);
        let encryption_domain_service = Arc::new(mock.encryption_domain_service);
        let message_archive_service = Arc::new(mock.message_archive_service);
        let messages_repo = Arc::new(mock.messages_repo);
        let messaging_service = Arc::new(mock.messaging_service);
        let room_management_service = Arc::new(mock.room_management_service);
        let room_participation_service = Arc::new(mock.room_participation_service);
        let room_attributes_service = Arc::new(mock.room_attributes_service);
        let sidebar_domain_service = Arc::new(mock.sidebar_domain_service);
        let user_info_domain_service = Arc::new(mock.user_info_domain_service);
        let synced_room_settings_service = Arc::new(mock.synced_room_settings_service);

        let room_factory = {
            let client_event_dispatcher = client_event_dispatcher.clone();
            let ctx = ctx.clone();
            let drafts_repo = drafts_repo.clone();
            let encryption_domain_service = encryption_domain_service.clone();
            let message_id_provider = mock.message_id_provider.clone();
            let message_archive_service = message_archive_service.clone();
            let message_repo = messages_repo.clone();
            let messaging_service = messaging_service.clone();
            let participation_service = room_participation_service.clone();
            let sidebar_domain_service = sidebar_domain_service.clone();
            let time_provider = mock.time_provider.clone();
            let topic_service = room_attributes_service.clone();
            let user_info_domain_service = user_info_domain_service.clone();
            let synced_room_settings_service = synced_room_settings_service.clone();

            RoomFactory::new(Arc::new(move |data| {
                RoomInner {
                    attributes_service: topic_service.clone(),
                    client_event_dispatcher: client_event_dispatcher.clone(),
                    ctx: ctx.clone(),
                    data: data.clone(),
                    drafts_repo: drafts_repo.clone(),
                    encryption_domain_service: encryption_domain_service.clone(),
                    message_id_provider: message_id_provider.clone(),
                    message_archive_service: message_archive_service.clone(),
                    message_repo: message_repo.clone(),
                    messaging_service: messaging_service.clone(),
                    participation_service: participation_service.clone(),
                    synced_room_settings_service: synced_room_settings_service.clone(),
                    sidebar_domain_service: sidebar_domain_service.clone(),
                    time_provider: time_provider.clone(),
                    user_info_domain_service: user_info_domain_service.clone(),
                }
                .into()
            }))
        };

        AppDependencies {
            account_settings_repo: Arc::new(mock.account_settings_repo),
            avatar_repo: Arc::new(mock.avatar_repo),
            block_list_domain_service: Arc::new(mock.block_list_domain_service),
            client_event_dispatcher,
            connected_rooms_repo,
            connection_service: Arc::new(mock.connection_service),
            ctx,
            drafts_repo,
            encryption_domain_service,
            id_provider: mock.id_provider,
            message_id_provider: mock.message_id_provider,
            local_room_settings_repo: Arc::new(mock.local_room_settings_repo),
            message_archive_service,
            messages_repo,
            messaging_service,
            offline_messages_repo: Arc::new(mock.offline_message_repo),
            request_handling_service: Arc::new(mock.request_handling_service),
            room_factory,
            room_management_service,
            room_participation_service,
            room_attributes_service,
            rooms_domain_service: Arc::new(mock.rooms_domain_service),
            server_event_handler_queue: Arc::new(ServerEventHandlerQueue::new()),
            short_id_provider: mock.short_id_provider,
            sidebar_domain_service,
            time_provider: mock.time_provider,
            upload_service: Arc::new(mock.upload_service),
            user_account_service: Arc::new(mock.user_account_service),
            user_device_repo: Arc::new(mock.user_device_repo),
            user_info_domain_service,
            contact_list_domain_service: Arc::new(mock.contact_list_domain_service),
            rng_provider: Arc::new(OsRngProvider),
        }
    }
}

#[derive(Default)]
pub struct MockSidebarDomainServiceDependencies {
    pub bookmarks_service: MockBookmarksService,
    pub client_event_dispatcher: MockClientEventDispatcherTrait,
    pub connected_rooms_repo: MockConnectedRoomsReadWriteRepository,
    pub ctx: AppContext,
    pub room_management_service: MockRoomManagementService,
    pub rooms_domain_service: MockRoomsDomainService,
}

impl MockSidebarDomainServiceDependencies {
    pub fn into_deps(self) -> SidebarDomainServiceDependencies {
        SidebarDomainServiceDependencies::from(self)
    }
}

impl From<MockSidebarDomainServiceDependencies> for SidebarDomainServiceDependencies {
    fn from(value: MockSidebarDomainServiceDependencies) -> Self {
        Self {
            bookmarks_service: Arc::new(value.bookmarks_service),
            client_event_dispatcher: Arc::new(value.client_event_dispatcher),
            connected_rooms_repo: Arc::new(value.connected_rooms_repo),
            ctx: Arc::new(value.ctx),
            room_management_service: Arc::new(value.room_management_service),
            rooms_domain_service: Arc::new(value.rooms_domain_service),
        }
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct MockRoomsDomainServiceDependencies {
    pub account_settings_repo: MockAccountSettingsRepository,
    pub client_event_dispatcher: MockClientEventDispatcherTrait,
    pub connected_rooms_repo: MockConnectedRoomsReadWriteRepository,
    pub ctx: AppContext,
    pub encryption_domain_service: MockEncryptionDomainService,
    #[derivative(Default(value = "Arc::new(IncrementingIDProvider::new(\"short-id\"))"))]
    pub id_provider: DynIDProvider,
    pub message_archive_domain_service: MockMessageArchiveDomainService,
    pub message_migration_domain_service: MockMessageMigrationDomainService,
    pub room_attributes_service: MockRoomAttributesService,
    pub room_management_service: MockRoomManagementService,
    pub room_participation_service: MockRoomParticipationService,
    pub synced_room_settings_service: MockSyncedRoomSettingsService,
    pub user_info_domain_service: MockUserInfoDomainService,
}

impl MockRoomsDomainServiceDependencies {
    pub fn into_deps(self) -> RoomsDomainServiceDependencies {
        RoomsDomainServiceDependencies::from(self)
    }
}

impl From<MockRoomsDomainServiceDependencies> for RoomsDomainServiceDependencies {
    fn from(value: MockRoomsDomainServiceDependencies) -> Self {
        Self {
            account_settings_repo: Arc::new(value.account_settings_repo),
            client_event_dispatcher: Arc::new(value.client_event_dispatcher),
            connected_rooms_repo: Arc::new(value.connected_rooms_repo),
            ctx: Arc::new(value.ctx),
            encryption_domain_service: Arc::new(value.encryption_domain_service),
            id_provider: Arc::new(value.id_provider),
            message_migration_domain_service: Arc::new(value.message_migration_domain_service),
            room_attributes_service: Arc::new(value.room_attributes_service),
            room_management_service: Arc::new(value.room_management_service),
            room_participation_service: Arc::new(value.room_participation_service),
            synced_room_settings_service: Arc::new(value.synced_room_settings_service),
            user_info_domain_service: Arc::new(value.user_info_domain_service),
            message_archive_domain_service: Arc::new(value.message_archive_domain_service),
        }
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct MockUserInfoDomainServiceDependencies {
    pub avatar_repo: MockAvatarRepository,
    pub block_list_repo: MockBlockListRepository,
    pub client_event_dispatcher: MockClientEventDispatcherTrait,
    pub ctx: AppContext,
    #[derivative(Default(value = "Arc::new(ConstantTimeProvider::new(mock_reference_date()))"))]
    pub time_provider: DynTimeProvider,
    pub user_info_repo: MockUserInfoRepository,
    pub user_info_service: MockUserInfoService,
    pub user_profile_repo: MockUserProfileRepository,
}

impl MockUserInfoDomainServiceDependencies {
    pub fn into_deps(self) -> UserInfoDomainServiceDependencies {
        UserInfoDomainServiceDependencies::from(self)
    }
}

impl From<MockUserInfoDomainServiceDependencies> for UserInfoDomainServiceDependencies {
    fn from(value: MockUserInfoDomainServiceDependencies) -> Self {
        Self {
            avatar_repo: Arc::new(value.avatar_repo),
            block_list_repo: Arc::new(value.block_list_repo),
            client_event_dispatcher: Arc::new(value.client_event_dispatcher),
            ctx: Arc::new(value.ctx),
            time_provider: Arc::new(value.time_provider),
            user_info_repo: Arc::new(value.user_info_repo),
            user_info_service: Arc::new(value.user_info_service),
            user_profile_repo: Arc::new(value.user_profile_repo),
        }
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct MockRoomFactoryDependencies {
    pub attributes_service: MockRoomAttributesService,
    pub bookmarks_service: MockBookmarksService,
    pub client_event_dispatcher: MockClientEventDispatcherTrait,
    pub ctx: AppContext,
    pub drafts_repo: MockDraftsRepository,
    pub encryption_domain_service: MockEncryptionDomainService,
    #[derivative(Default(
        value = "Arc::new(WrappingMessageIdProvider::incrementing(\"msg-id\"))"
    ))]
    pub message_id_provider: DynMessageIdProvider,
    pub message_archive_service: MockMessageArchiveService,
    pub message_repo: MockMessagesRepository,
    pub messaging_service: MockMessagingService,
    pub participation_service: MockRoomParticipationService,
    pub synced_room_settings_service: MockSyncedRoomSettingsService,
    pub sidebar_domain_service: MockSidebarDomainService,
    #[derivative(Default(value = "Arc::new(ConstantTimeProvider::new(mock_reference_date()))"))]
    pub time_provider: DynTimeProvider,
    pub user_info_domain_service: MockUserInfoDomainService,
}

pub struct MockSealedRoomFactoryDependencies {
    pub bookmarks_service: DynBookmarksService,
    pub client_event_dispatcher: DynClientEventDispatcher,
    pub ctx: DynAppContext,
    pub drafts_repo: DynDraftsRepository,
    pub encryption_domain_service: DynEncryptionDomainService,
    pub message_id_provider: DynMessageIdProvider,
    pub message_archive_service: DynMessageArchiveService,
    pub message_repo: DynMessagesRepository,
    pub messaging_service: DynMessagingService,
    pub participation_service: DynRoomParticipationService,
    pub synced_room_settings_service: DynSyncedRoomSettingsService,
    pub sidebar_domain_service: DynSidebarDomainService,
    pub time_provider: DynTimeProvider,
    pub topic_service: DynRoomAttributesService,
    pub user_info_domain_service: DynUserInfoDomainService,
}

impl From<MockRoomFactoryDependencies> for MockSealedRoomFactoryDependencies {
    fn from(value: MockRoomFactoryDependencies) -> Self {
        Self {
            bookmarks_service: Arc::new(value.bookmarks_service),
            client_event_dispatcher: Arc::new(value.client_event_dispatcher),
            ctx: Arc::new(value.ctx),
            drafts_repo: Arc::new(value.drafts_repo),
            encryption_domain_service: Arc::new(value.encryption_domain_service),
            message_id_provider: value.message_id_provider,
            message_archive_service: Arc::new(value.message_archive_service),
            message_repo: Arc::new(value.message_repo),
            messaging_service: Arc::new(value.messaging_service),
            participation_service: Arc::new(value.participation_service),
            synced_room_settings_service: Arc::new(value.synced_room_settings_service),
            sidebar_domain_service: Arc::new(value.sidebar_domain_service),
            time_provider: Arc::new(value.time_provider),
            topic_service: Arc::new(value.attributes_service),
            user_info_domain_service: Arc::new(value.user_info_domain_service),
        }
    }
}

impl From<MockSealedRoomFactoryDependencies> for RoomFactory {
    fn from(value: MockSealedRoomFactoryDependencies) -> Self {
        RoomFactory::new(Arc::new(move |data| {
            RoomInner {
                attributes_service: value.topic_service.clone(),
                client_event_dispatcher: value.client_event_dispatcher.clone(),
                ctx: value.ctx.clone(),
                data: data.clone(),
                drafts_repo: value.drafts_repo.clone(),
                encryption_domain_service: value.encryption_domain_service.clone(),
                message_id_provider: value.message_id_provider.clone(),
                message_archive_service: value.message_archive_service.clone(),
                message_repo: value.message_repo.clone(),
                messaging_service: value.messaging_service.clone(),
                participation_service: value.participation_service.clone(),
                synced_room_settings_service: value.synced_room_settings_service.clone(),
                sidebar_domain_service: value.sidebar_domain_service.clone(),
                time_provider: value.time_provider.clone(),
                user_info_domain_service: value.user_info_domain_service.clone(),
            }
            .into()
        }))
    }
}

impl From<MockRoomFactoryDependencies> for RoomFactory {
    fn from(value: MockRoomFactoryDependencies) -> Self {
        MockSealedRoomFactoryDependencies::from(value).into()
    }
}

impl RoomFactory {
    pub fn mock() -> Self {
        RoomFactory::from(MockSealedRoomFactoryDependencies::from(
            MockRoomFactoryDependencies::default(),
        ))
    }
}
