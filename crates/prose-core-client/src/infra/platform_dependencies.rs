// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use prose_store::prelude::*;

use crate::app::deps::{
    AppContext, AppDependencies, DynClientEventDispatcher, DynIDProvider, DynTimeProvider,
};
use crate::app::services::RoomInner;
use crate::domain::contacts::services::impls::{
    BlockListDomainService, BlockListDomainServiceDependencies, ContactListDomainService,
    ContactListDomainServiceDependencies,
};
use crate::domain::messaging::services::impls::{
    MessageMigrationDomainService, MessageMigrationDomainServiceDependencies,
};
use crate::domain::rooms::services::impls::{RoomsDomainService, RoomsDomainServiceDependencies};
use crate::domain::rooms::services::RoomFactory;
use crate::domain::sidebar::services::impls::{
    SidebarDomainService, SidebarDomainServiceDependencies,
};
use crate::infra::avatars::AvatarCache;
use crate::infra::contacts::{
    CachingBlockListRepository, CachingContactsRepository, PresenceSubRequestsRepository,
};
use crate::infra::messaging::{
    CachingMessageRepository, DraftsRecord, DraftsRepository, MessageRecord,
};
use crate::infra::rooms::InMemoryConnectedRoomsRepository;
use crate::infra::settings::{AccountSettingsRecord, AccountSettingsRepository};
use crate::infra::user_info::caching_avatar_repository::CachingAvatarRepository;
use crate::infra::user_info::{CachingUserInfoRepository, UserInfoRecord};
use crate::infra::user_profile::{CachingUserProfileRepository, UserProfileRecord};
use crate::infra::xmpp::XMPPClient;

pub(crate) struct PlatformDependencies {
    pub avatar_cache: Box<dyn AvatarCache>,
    pub client_event_dispatcher: DynClientEventDispatcher,
    pub ctx: AppContext,
    pub id_provider: DynIDProvider,
    pub short_id_provider: DynIDProvider,
    pub store: Store<PlatformDriver>,
    pub time_provider: DynTimeProvider,
    pub xmpp: Arc<XMPPClient>,
}

pub async fn open_store<D: Driver>(driver: D) -> Result<Store<D>, D::Error> {
    let versions_changed = Arc::new(AtomicBool::new(false));

    let inner_versions_changed = versions_changed.clone();
    let store = Store::open(driver, 15, move |event| {
        let tx = &event.tx;

        inner_versions_changed.store(true, Ordering::Relaxed);

        if event.old_version < 10 {
            create_collection::<D, AccountSettingsRecord>(&tx)?;
            create_collection::<D, DraftsRecord>(&tx)?;
            create_collection::<D, MessageRecord>(&tx)?;
            create_collection::<D, UserInfoRecord>(&tx)?;
            create_collection::<D, UserProfileRecord>(&tx)?;
            #[cfg(target_arch = "wasm32")]
            create_collection::<D, crate::infra::avatars::AvatarRecord>(&tx)?;
        }

        if event.old_version < 13 {
            tx.delete_collection(MessageRecord::collection())?;
        }

        if event.old_version < 14 {
            create_collection::<D, MessageRecord>(&tx)?;
        }

        if event.old_version < 15 {
            tx.delete_collection(MessageRecord::collection())?;
            create_collection::<D, MessageRecord>(&tx)?;
        }

        Ok(())
    })
    .await?;

    if versions_changed.load(Ordering::Acquire) {
        store
            .truncate_collections(&[
                MessageRecord::collection(),
                UserInfoRecord::collection(),
                UserProfileRecord::collection(),
                #[cfg(target_arch = "wasm32")]
                crate::infra::avatars::AvatarRecord::collection(),
            ])
            .await?;
    }

    Ok(store)
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
        let account_settings_repo = Arc::new(AccountSettingsRepository::new(d.store.clone()));
        let client_event_dispatcher = d.client_event_dispatcher;
        let connected_rooms_repo = Arc::new(InMemoryConnectedRoomsRepository::new());
        let ctx = Arc::new(d.ctx);
        let drafts_repo = Arc::new(DraftsRepository::new(d.store.clone()));
        let id_provider = d.id_provider;
        let messages_repo = Arc::new(CachingMessageRepository::new(d.store.clone()));
        let time_provider = d.time_provider;
        let user_info_repo = Arc::new(CachingUserInfoRepository::new(
            d.store.clone(),
            d.xmpp.clone(),
        ));
        let user_profile_repo = Arc::new(CachingUserProfileRepository::new(
            d.store.clone(),
            d.xmpp.clone(),
        ));

        let message_migration_domain_service_dependencies =
            MessageMigrationDomainServiceDependencies {
                message_archive_service: d.xmpp.clone(),
                messaging_service: d.xmpp.clone(),
            };

        let message_migration_domain_service = Arc::new(MessageMigrationDomainService::from(
            message_migration_domain_service_dependencies,
        ));

        let rooms_domain_service_dependencies = RoomsDomainServiceDependencies {
            account_settings_repo: account_settings_repo.clone(),
            client_event_dispatcher: client_event_dispatcher.clone(),
            connected_rooms_repo: connected_rooms_repo.clone(),
            ctx: ctx.clone(),
            id_provider: d.short_id_provider.clone(),
            message_migration_domain_service: message_migration_domain_service.clone(),
            room_attributes_service: d.xmpp.clone(),
            room_management_service: d.xmpp.clone(),
            room_participation_service: d.xmpp.clone(),
            user_info_repo: user_info_repo.clone(),
            user_profile_repo: user_profile_repo.clone(),
        };

        let rooms_domain_service =
            Arc::new(RoomsDomainService::from(rooms_domain_service_dependencies));

        let sidebar_domain_service_dependencies = SidebarDomainServiceDependencies {
            bookmarks_service: d.xmpp.clone(),
            client_event_dispatcher: client_event_dispatcher.clone(),
            connected_rooms_repo: connected_rooms_repo.clone(),
            ctx: ctx.clone(),
            room_management_service: d.xmpp.clone(),
            rooms_domain_service: rooms_domain_service.clone(),
        };

        let sidebar_domain_service = Arc::new(SidebarDomainService::from(
            sidebar_domain_service_dependencies,
        ));

        let contact_list_domain_service_dependencies = ContactListDomainServiceDependencies {
            client_event_dispatcher: client_event_dispatcher.clone(),
            contact_list_repo: Arc::new(CachingContactsRepository::new(d.xmpp.clone())),
            contact_list_service: d.xmpp.clone(),
            presence_sub_requests_repo: Arc::new(PresenceSubRequestsRepository::new()),
        };

        let contact_list_domain_service = Arc::new(ContactListDomainService::from(
            contact_list_domain_service_dependencies,
        ));

        let block_list_domain_service_dependencies = BlockListDomainServiceDependencies {
            block_list_repo: Arc::new(CachingBlockListRepository::new(d.xmpp.clone())),
            block_list_service: d.xmpp.clone(),
            client_event_dispatcher: client_event_dispatcher.clone(),
        };

        let block_list_domain_service = Arc::new(BlockListDomainService::from(
            block_list_domain_service_dependencies,
        ));

        let room_factory = {
            let client_event_dispatcher = client_event_dispatcher.clone();
            let ctx = ctx.clone();
            let xmpp = d.xmpp.clone();
            let time_provider = time_provider.clone();
            let message_repo = messages_repo.clone();
            let drafts_repo = drafts_repo.clone();
            let user_profile_repo = user_profile_repo.clone();
            let sidebar_domain_service = sidebar_domain_service.clone();

            RoomFactory::new(Arc::new(move |data| {
                RoomInner {
                    data: data.clone(),
                    ctx: ctx.clone(),
                    time_provider: time_provider.clone(),
                    messaging_service: xmpp.clone(),
                    message_archive_service: xmpp.clone(),
                    participation_service: xmpp.clone(),
                    attributes_service: xmpp.clone(),
                    message_repo: message_repo.clone(),
                    drafts_repo: drafts_repo.clone(),
                    user_profile_repo: user_profile_repo.clone(),
                    client_event_dispatcher: client_event_dispatcher.clone(),
                    sidebar_domain_service: sidebar_domain_service.clone(),
                }
                .into()
            }))
        };

        Self {
            account_settings_repo,
            avatar_repo: Arc::new(CachingAvatarRepository::new(d.xmpp.clone(), d.avatar_cache)),
            block_list_domain_service,
            client_event_dispatcher,
            connected_rooms_repo,
            connection_service: d.xmpp.clone(),
            contact_list_domain_service,
            ctx,
            drafts_repo,
            request_handling_service: d.xmpp.clone(),
            room_factory,
            room_management_service: d.xmpp.clone(),
            message_archive_service: d.xmpp.clone(),
            messages_repo,
            messaging_service: d.xmpp.clone(),
            room_participation_service: d.xmpp.clone(),
            room_attributes_service: d.xmpp.clone(),
            rooms_domain_service,
            short_id_provider: d.short_id_provider,
            sidebar_domain_service,
            time_provider,
            upload_service: d.xmpp.clone(),
            user_account_service: d.xmpp.clone(),
            user_info_repo,
            user_info_service: d.xmpp.clone(),
            user_profile_repo,
            user_profile_service: d.xmpp.clone(),
            id_provider,
        }
    }
}
