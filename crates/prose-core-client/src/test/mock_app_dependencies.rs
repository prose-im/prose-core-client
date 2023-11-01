// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::convert::Into;
use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use derivative::Derivative;
use jid::{BareJid, FullJid};
use parking_lot::RwLock;

use prose_xmpp::test::IncrementingIDProvider;
use prose_xmpp::{bare, full};

use crate::app::deps::{
    AppContext, AppDependencies, DynDraftsRepository, DynIDProvider, DynMessageArchiveService,
    DynMessagesRepository, DynMessagingService, DynRoomParticipationService, DynRoomTopicService,
    DynTimeProvider,
};
use crate::app::event_handlers::MockEventDispatcher;
use crate::app::services::RoomInner;
use crate::domain::account::services::mocks::MockUserAccountService;
use crate::domain::connection::models::{ConnectionProperties, ServerFeatures};
use crate::domain::connection::services::mocks::MockConnectionService;
use crate::domain::contacts::repos::mocks::MockContactsRepository;
use crate::domain::contacts::services::mocks::MockContactsService;
use crate::domain::general::models::Capabilities;
use crate::domain::general::services::mocks::MockRequestHandlingService;
use crate::domain::messaging::repos::mocks::{MockDraftsRepository, MockMessagesRepository};
use crate::domain::messaging::services::mocks::{MockMessageArchiveService, MockMessagingService};
use crate::domain::rooms::repos::mocks::{MockBookmarksRepository, MockConnectedRoomsRepository};
use crate::domain::rooms::services::mocks::{
    MockRoomManagementService, MockRoomParticipationService, MockRoomTopicService,
    MockRoomsDomainService,
};
use crate::domain::rooms::services::RoomFactory;
use crate::domain::settings::repos::mocks::MockAccountSettingsRepository;
use crate::domain::user_info::repos::mocks::{MockAvatarRepository, MockUserInfoRepository};
use crate::domain::user_info::services::mocks::MockUserInfoService;
use crate::domain::user_profiles::repos::mocks::MockUserProfileRepository;
use crate::domain::user_profiles::services::mocks::MockUserProfileService;
use crate::test::ConstantTimeProvider;
use crate::ClientEvent;

pub fn mock_reference_date() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 09, 06, 0, 0, 0).unwrap().into()
}

pub fn mock_muc_service() -> BareJid {
    bare!("conference.prose.org")
}

pub fn mock_account_jid() -> FullJid {
    full!("jane.doe@prose.org/macOS")
}

impl Default for AppContext {
    fn default() -> Self {
        AppContext {
            connection_properties: RwLock::new(Some(ConnectionProperties {
                connected_jid: mock_account_jid(),
                server_features: ServerFeatures {
                    muc_service: Some(mock_muc_service()),
                },
            })),
            capabilities: Capabilities::new("Prose", "https://prose.org", vec![]),
            software_version: Default::default(),
            is_observing_rooms: Default::default(),
        }
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct MockAppDependencies {
    pub account_settings_repo: MockAccountSettingsRepository,
    pub avatar_repo: MockAvatarRepository,
    pub bookmarks_repo: MockBookmarksRepository,
    pub client_event_dispatcher: MockEventDispatcher<ClientEvent>,
    pub connected_rooms_repo: MockConnectedRoomsRepository,
    pub connection_service: MockConnectionService,
    pub contacts_repo: MockContactsRepository,
    pub contacts_service: MockContactsService,
    pub ctx: AppContext,
    pub drafts_repo: MockDraftsRepository,
    #[derivative(Default(value = "Arc::new(IncrementingIDProvider::new(\"id\"))"))]
    pub id_provider: DynIDProvider,
    pub message_archive_service: MockMessageArchiveService,
    pub messages_repo: MockMessagesRepository,
    pub messaging_service: MockMessagingService,
    pub request_handling_service: MockRequestHandlingService,
    pub rooms_domain_service: MockRoomsDomainService,
    pub room_management_service: MockRoomManagementService,
    pub room_participation_service: MockRoomParticipationService,
    pub room_topic_service: MockRoomTopicService,
    #[derivative(Default(value = "Arc::new(IncrementingIDProvider::new(\"short-id\"))"))]
    pub short_id_provider: DynIDProvider,
    #[derivative(Default(value = "Arc::new(ConstantTimeProvider::new(mock_reference_date()))"))]
    pub time_provider: DynTimeProvider,
    pub user_account_service: MockUserAccountService,
    pub user_info_repo: MockUserInfoRepository,
    pub user_info_service: MockUserInfoService,
    pub user_profile_repo: MockUserProfileRepository,
    pub user_profile_service: MockUserProfileService,
}

impl MockAppDependencies {
    pub fn into_deps(self) -> AppDependencies {
        AppDependencies::from(self)
    }
}

impl From<MockAppDependencies> for AppDependencies {
    fn from(mock: MockAppDependencies) -> Self {
        let bookmarks_repo = Arc::new(mock.bookmarks_repo);
        let client_event_dispatcher = Arc::new(mock.client_event_dispatcher);
        let connected_rooms_repo = Arc::new(mock.connected_rooms_repo);
        let ctx = Arc::new(mock.ctx);
        let drafts_repo = Arc::new(mock.drafts_repo);
        let message_archive_service = Arc::new(mock.message_archive_service);
        let messages_repo = Arc::new(mock.messages_repo);
        let messaging_service = Arc::new(mock.messaging_service);
        let room_management_service = Arc::new(mock.room_management_service);
        let room_participation_service = Arc::new(mock.room_participation_service);
        let room_topic_service = Arc::new(mock.room_topic_service);
        let user_profile_repo = Arc::new(mock.user_profile_repo);

        let room_factory = {
            let drafts_repo = drafts_repo.clone();
            let message_archive_service = message_archive_service.clone();
            let message_repo = messages_repo.clone();
            let messaging_service = messaging_service.clone();
            let participation_service = room_participation_service.clone();
            let time_provider = mock.time_provider.clone();
            let topic_service = room_topic_service.clone();

            RoomFactory::new(Arc::new(move |data| {
                RoomInner {
                    data: data.clone(),
                    time_provider: time_provider.clone(),
                    messaging_service: messaging_service.clone(),
                    message_archive_service: message_archive_service.clone(),
                    participation_service: participation_service.clone(),
                    topic_service: topic_service.clone(),
                    message_repo: message_repo.clone(),
                    drafts_repo: drafts_repo.clone(),
                }
                .into()
            }))
        };

        AppDependencies {
            account_settings_repo: Arc::new(mock.account_settings_repo),
            avatar_repo: Arc::new(mock.avatar_repo),
            bookmarks_repo,
            client_event_dispatcher,
            connected_rooms_repo,
            connection_service: Arc::new(mock.connection_service),
            contacts_repo: Arc::new(mock.contacts_repo),
            contacts_service: Arc::new(mock.contacts_service),
            ctx,
            drafts_repo,
            id_provider: mock.id_provider,
            message_archive_service,
            messages_repo,
            messaging_service,
            request_handling_service: Arc::new(mock.request_handling_service),
            room_factory,
            room_management_service,
            room_participation_service,
            room_topic_service,
            rooms_domain_service: Arc::new(mock.rooms_domain_service),
            short_id_provider: mock.short_id_provider,
            time_provider: mock.time_provider,
            user_account_service: Arc::new(mock.user_account_service),
            user_info_repo: Arc::new(mock.user_info_repo),
            user_info_service: Arc::new(mock.user_info_service),
            user_profile_repo,
            user_profile_service: Arc::new(mock.user_profile_service),
        }
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct MockRoomFactoryDependencies {
    pub drafts_repo: MockDraftsRepository,
    pub message_archive_service: MockMessageArchiveService,
    pub message_repo: MockMessagesRepository,
    pub messaging_service: MockMessagingService,
    pub participation_service: MockRoomParticipationService,
    #[derivative(Default(value = "Arc::new(ConstantTimeProvider::new(mock_reference_date()))"))]
    pub time_provider: DynTimeProvider,
    pub topic_service: MockRoomTopicService,
}

pub struct MockSealedRoomFactoryDependencies {
    pub drafts_repo: DynDraftsRepository,
    pub message_archive_service: DynMessageArchiveService,
    pub message_repo: DynMessagesRepository,
    pub messaging_service: DynMessagingService,
    pub participation_service: DynRoomParticipationService,
    pub time_provider: DynTimeProvider,
    pub topic_service: DynRoomTopicService,
}

impl From<MockRoomFactoryDependencies> for MockSealedRoomFactoryDependencies {
    fn from(value: MockRoomFactoryDependencies) -> Self {
        Self {
            drafts_repo: Arc::new(value.drafts_repo),
            message_archive_service: Arc::new(value.message_archive_service),
            message_repo: Arc::new(value.message_repo),
            messaging_service: Arc::new(value.messaging_service),
            participation_service: Arc::new(value.participation_service),
            time_provider: Arc::new(value.time_provider),
            topic_service: Arc::new(value.topic_service),
        }
    }
}

impl From<MockSealedRoomFactoryDependencies> for RoomFactory {
    fn from(value: MockSealedRoomFactoryDependencies) -> Self {
        RoomFactory::new(Arc::new(move |data| {
            RoomInner {
                data: data.clone(),
                time_provider: value.time_provider.clone(),
                messaging_service: value.messaging_service.clone(),
                message_archive_service: value.message_archive_service.clone(),
                participation_service: value.participation_service.clone(),
                topic_service: value.topic_service.clone(),
                message_repo: value.message_repo.clone(),
                drafts_repo: value.drafts_repo.clone(),
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
