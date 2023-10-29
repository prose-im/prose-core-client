// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use prose_store::prelude::*;

use crate::app::deps::{AppContext, AppDependencies, AppServiceDependencies};
use crate::app::services::RoomInner;
use crate::domain::rooms::services::{RoomFactory, RoomsDomainService};
use crate::infra::avatars::AvatarCache;
use crate::infra::contacts::CachingContactsRepository;
use crate::infra::messaging::{
    CachingMessageRepository, DraftsRecord, DraftsRepository, MessagesRecord,
};
use crate::infra::rooms::{CachingBookmarksRepository, InMemoryConnectedRoomsRepository};
use crate::infra::settings::{AccountSettingsRecord, AccountSettingsRepository};
use crate::infra::user_info::caching_avatar_repository::CachingAvatarRepository;
use crate::infra::user_info::{CachingUserInfoRepository, UserInfoRecord};
use crate::infra::user_profile::{CachingUserProfileRepository, UserProfileRecord};
use crate::infra::xmpp::XMPPClient;

pub(crate) struct PlatformDependencies {
    pub ctx: AppContext,
    pub app_service_deps: AppServiceDependencies,
    pub store: Store<PlatformDriver>,
    pub xmpp: Arc<XMPPClient>,
    pub avatar_cache: Box<dyn AvatarCache>,
}

pub async fn open_store<D: Driver>(driver: D) -> Result<Store<D>, D::Error> {
    Store::open(driver, 10, |event| {
        let tx = &event.tx;

        create_collection::<D, AccountSettingsRecord>(&tx)?;
        create_collection::<D, DraftsRecord>(&tx)?;
        create_collection::<D, MessagesRecord>(&tx)?;
        create_collection::<D, UserInfoRecord>(&tx)?;
        create_collection::<D, UserProfileRecord>(&tx)?;

        Ok(())
    })
    .await
}

fn create_collection<D: Driver, E: Entity>(tx: &D::UpgradeTransaction<'_>) -> Result<(), D::Error> {
    let collection = tx.create_collection(E::collection())?;
    for idx_spec in E::indexes() {
        collection.add_index(idx_spec)?;
    }
    Ok(())
}

impl From<PlatformDependencies> for AppDependencies {
    fn from(d: PlatformDependencies) -> Self {
        let app_service = Arc::new(d.app_service_deps);
        let bookmarks_repo = Arc::new(CachingBookmarksRepository::new(d.xmpp.clone()));
        let connected_rooms_repo = Arc::new(InMemoryConnectedRoomsRepository::new());
        let ctx = Arc::new(d.ctx);
        let drafts_repo = Arc::new(DraftsRepository::new(d.store.clone()));
        let messages_repo = Arc::new(CachingMessageRepository::new(d.store.clone()));
        let user_profile_repo = Arc::new(CachingUserProfileRepository::new(
            d.store.clone(),
            d.xmpp.clone(),
        ));

        let room_factory = {
            let xmpp = d.xmpp.clone();
            let deps = app_service.clone();
            let message_repo = messages_repo.clone();
            let drafts_repo = drafts_repo.clone();

            RoomFactory::new(Arc::new(move |data| {
                RoomInner {
                    data: data.clone(),
                    deps: deps.clone(),
                    messaging_service: xmpp.clone(),
                    message_archive_service: xmpp.clone(),
                    participation_service: xmpp.clone(),
                    topic_service: xmpp.clone(),
                    message_repo: message_repo.clone(),
                    drafts_repo: drafts_repo.clone(),
                }
                .into()
            }))
        };

        let rooms_domain_service = RoomsDomainService {
            app_service: app_service.clone(),
            bookmarks_repo: bookmarks_repo.clone(),
            connected_rooms_repo: connected_rooms_repo.clone(),
            ctx: ctx.clone(),
            room_management_service: d.xmpp.clone(),
            room_participation_service: d.xmpp.clone(),
            user_profile_repo: user_profile_repo.clone(),
        };

        Self {
            account_settings_repo: Arc::new(AccountSettingsRepository::new(d.store.clone())),
            app_service,
            avatar_repo: Arc::new(CachingAvatarRepository::new(d.xmpp.clone(), d.avatar_cache)),
            bookmarks_repo,
            connected_rooms_repo,
            connection_service: d.xmpp.clone(),
            contacts_repo: Arc::new(CachingContactsRepository::new(d.xmpp.clone())),
            contacts_service: d.xmpp.clone(),
            ctx,
            drafts_repo,
            request_handling_service: d.xmpp.clone(),
            room_factory,
            room_management_service: d.xmpp.clone(),
            message_archive_service: d.xmpp.clone(),
            messages_repo,
            messaging_service: d.xmpp.clone(),
            room_participation_service: d.xmpp.clone(),
            room_topic_service: d.xmpp.clone(),
            rooms_domain_service: Arc::new(rooms_domain_service),
            user_account_service: d.xmpp.clone(),
            user_info_repo: Arc::new(CachingUserInfoRepository::new(
                d.store.clone(),
                d.xmpp.clone(),
            )),
            user_info_service: d.xmpp.clone(),
            user_profile_repo,
            user_profile_service: d.xmpp.clone(),
        }
    }
}
