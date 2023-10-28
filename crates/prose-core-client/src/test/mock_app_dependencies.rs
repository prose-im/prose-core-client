// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::convert::Into;
use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use jid::{BareJid, FullJid};
use parking_lot::RwLock;

use prose_xmpp::test::IncrementingIDProvider;
use prose_xmpp::{bare, full};

use crate::app::deps::{AppContext, AppDependencies, AppServiceDependencies};
use crate::app::event_handlers::ClientEventDispatcher;
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
use crate::domain::rooms::repos::mocks::MockBookmarksRepository;
use crate::domain::rooms::services::mocks::{
    MockRoomManagementService, MockRoomParticipationService, MockRoomTopicService,
};
use crate::domain::rooms::services::RoomFactory;
use crate::domain::settings::repos::mocks::MockAccountSettingsRepository;
use crate::domain::user_info::repos::mocks::{MockAvatarRepository, MockUserInfoRepository};
use crate::domain::user_info::services::mocks::MockUserInfoService;
use crate::domain::user_profiles::repos::mocks::MockUserProfileRepository;
use crate::domain::user_profiles::services::mocks::MockUserProfileService;
use crate::test::ConstantTimeProvider;

pub fn mock_reference_date() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2021, 09, 06, 0, 0, 0).unwrap().into()
}

pub fn mock_muc_service() -> BareJid {
    bare!("conference.prose.org")
}

pub fn mock_account_jid() -> FullJid {
    full!("jane.doe@prose.org/macOS")
}

impl Default for AppServiceDependencies {
    fn default() -> Self {
        Self {
            time_provider: Arc::new(ConstantTimeProvider {
                time: mock_reference_date(),
            }),
            id_provider: Arc::new(IncrementingIDProvider::new("id")),
            short_id_provider: Arc::new(IncrementingIDProvider::new("short-id")),
            event_dispatcher: Arc::new(ClientEventDispatcher::new(None)),
        }
    }
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

#[derive(Default)]
pub struct MockAppDependencies {
    pub account_settings_repo: MockAccountSettingsRepository,
    pub app_service: AppServiceDependencies,
    pub avatar_repo: MockAvatarRepository,
    pub bookmarks_repo: MockBookmarksRepository,
    pub connection_service: MockConnectionService,
    pub contacts_repo: MockContactsRepository,
    pub contacts_service: MockContactsService,
    pub ctx: AppContext,
    pub drafts_repo: MockDraftsRepository,
    pub messages_repo: MockMessagesRepository,
    pub request_handling_service: MockRequestHandlingService,
    pub room_management_service: MockRoomManagementService,
    pub message_archive_service: MockMessageArchiveService,
    pub messaging_service: MockMessagingService,
    pub room_participation_service: MockRoomParticipationService,
    pub room_topic_service: MockRoomTopicService,
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
        let drafts_repo = Arc::new(mock.drafts_repo);
        let app_service = Arc::new(mock.app_service);
        let messaging_service = Arc::new(mock.messaging_service);
        let message_archive_service = Arc::new(mock.message_archive_service);
        let room_participation_service = Arc::new(mock.room_participation_service);
        let room_topic_service = Arc::new(mock.room_topic_service);
        let messages_repo = Arc::new(mock.messages_repo);

        let room_factory = {
            let drafts_repo = drafts_repo.clone();
            let deps = app_service.clone();
            let messaging_service = messaging_service.clone();
            let message_archive_service = message_archive_service.clone();
            let participation_service = room_participation_service.clone();
            let topic_service = room_topic_service.clone();
            let message_repo = messages_repo.clone();

            RoomFactory::new(move |data| {
                RoomInner {
                    data: data.clone(),
                    deps: deps.clone(),
                    messaging_service: messaging_service.clone(),
                    message_archive_service: message_archive_service.clone(),
                    participation_service: participation_service.clone(),
                    topic_service: topic_service.clone(),
                    message_repo: message_repo.clone(),
                    drafts_repo: drafts_repo.clone(),
                }
                .into()
            })
        };

        AppDependencies {
            account_settings_repo: Arc::new(mock.account_settings_repo),
            app_service,
            avatar_repo: Arc::new(mock.avatar_repo),
            bookmarks_repo: Arc::new(mock.bookmarks_repo),
            connection_service: Arc::new(mock.connection_service),
            contacts_repo: Arc::new(mock.contacts_repo),
            contacts_service: Arc::new(mock.contacts_service),
            ctx: Arc::new(mock.ctx),
            drafts_repo,
            message_archive_service,
            messages_repo,
            messaging_service,
            request_handling_service: Arc::new(mock.request_handling_service),
            room_factory,
            room_management_service: Arc::new(mock.room_management_service),
            room_participation_service,
            room_topic_service,
            user_account_service: Arc::new(mock.user_account_service),
            user_info_repo: Arc::new(mock.user_info_repo),
            user_info_service: Arc::new(mock.user_info_service),
            user_profile_repo: Arc::new(mock.user_profile_repo),
            user_profile_service: Arc::new(mock.user_profile_service),
        }
    }
}
