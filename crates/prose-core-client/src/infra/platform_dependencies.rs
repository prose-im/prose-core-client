// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use prose_store::prelude::*;

use crate::app::deps::{
    AppContext, AppDependencies, DynClientEventDispatcher, DynEncryptionService, DynIDProvider,
    DynRngProvider, DynServerEventHandlerQueue, DynTimeProvider, DynUserDeviceIdProvider,
};
use crate::app::services::RoomInner;
use crate::domain::contacts::services::impls::{
    BlockListDomainService, BlockListDomainServiceDependencies, ContactListDomainService,
    ContactListDomainServiceDependencies,
};
use crate::domain::encryption::services::impls::{
    EncryptionDomainService, EncryptionDomainServiceDependencies,
};
use crate::domain::messaging::services::impls::{
    MessageArchiveDomainService, MessageArchiveDomainServiceDependencies,
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
use crate::infra::encryption::{
    CachingUserDeviceRepository, EncryptionKeysRepository, KyberPreKeyRecord, LocalDeviceRecord,
    PreKeyRecord, SenderKeyRecord, SessionRecord, SessionRepository, SignedPreKeyRecord,
    UserDeviceRecord,
};
use crate::infra::messaging::{
    CachingMessageRepository, DraftsRecord, DraftsRepository, MessageRecord,
    OfflineMessagesRepository,
};
use crate::infra::rooms::InMemoryConnectedRoomsRepository;
use crate::infra::settings::{
    AccountSettingsRecord, AccountSettingsRepository, LocalRoomSettingsRecord,
    LocalRoomSettingsRepository,
};
use crate::infra::user_info::caching_avatar_repository::CachingAvatarRepository;
use crate::infra::user_info::{CachingUserInfoRepository, UserInfoRecord};
use crate::infra::user_profile::{CachingUserProfileRepository, UserProfileRecord};
use crate::infra::xmpp::XMPPClient;

pub(crate) struct PlatformDependencies {
    pub avatar_cache: Box<dyn AvatarCache>,
    pub client_event_dispatcher: DynClientEventDispatcher,
    pub ctx: AppContext,
    pub encryption_service: DynEncryptionService,
    pub id_provider: DynIDProvider,
    pub rng_provider: DynRngProvider,
    pub server_event_handler_queue: DynServerEventHandlerQueue,
    pub short_id_provider: DynIDProvider,
    pub store: Store<PlatformDriver>,
    pub time_provider: DynTimeProvider,
    pub user_device_id_provider: DynUserDeviceIdProvider,
    pub xmpp: Arc<XMPPClient>,
}

const DB_VERSION: u32 = 27;

pub async fn open_store<D: Driver>(driver: D) -> Result<Store<D>, D::Error> {
    let versions_changed = Arc::new(AtomicBool::new(false));

    let inner_versions_changed = versions_changed.clone();
    let store = Store::open(driver, DB_VERSION, move |event| {
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

        if event.old_version < 16 {
            create_collection::<D, UserDeviceRecord>(&tx)?;
            tx.create_collection("omemo_identity")?;

            create_collection::<D, LocalDeviceRecord>(&tx)?;
            create_collection::<D, PreKeyRecord>(&tx)?;
            create_collection::<D, SenderKeyRecord>(&tx)?;
            create_collection::<D, SessionRecord>(&tx)?;
            create_collection::<D, SignedPreKeyRecord>(&tx)?;
        }

        if event.old_version < 17 {
            create_collection::<D, KyberPreKeyRecord>(&tx)?;
        }

        if event.old_version < 19 {
            tx.delete_collection("omemo_identity")?;
            tx.delete_collection(SessionRecord::collection())?;
            create_collection::<D, SessionRecord>(&tx)?;
        }

        if event.old_version < 20 {
            tx.delete_collection(MessageRecord::collection())?;
            create_collection::<D, MessageRecord>(&tx)?;
            tx.delete_collection(DraftsRecord::collection())?;
            create_collection::<D, DraftsRecord>(&tx)?;
        }

        if event.old_version < 21 {
            tx.delete_collection(MessageRecord::collection())?;
            create_collection::<D, MessageRecord>(&tx)?;
        }

        if event.old_version < 22 {
            tx.delete_collection(MessageRecord::collection())?;
            create_collection::<D, MessageRecord>(&tx)?;
            tx.delete_collection(UserDeviceRecord::collection())?;
            create_collection::<D, UserDeviceRecord>(&tx)?;
        }

        if event.old_version < 23 {
            create_collection::<D, LocalRoomSettingsRecord>(&tx)?;
        }

        if event.old_version < 24 {
            tx.delete_collection(MessageRecord::collection())?;
            create_collection::<D, MessageRecord>(&tx)?;
        }

        if event.old_version < 26 {
            tx.delete_collection(MessageRecord::collection())?;
            create_collection::<D, MessageRecord>(&tx)?;
        }

        if event.old_version < 27 {
            tx.delete_collection(LocalDeviceRecord::collection())?;
            tx.delete_collection(PreKeyRecord::collection())?;
            tx.delete_collection(SenderKeyRecord::collection())?;
            tx.delete_collection(SessionRecord::collection())?;
            tx.delete_collection(SignedPreKeyRecord::collection())?;
            tx.delete_collection(KyberPreKeyRecord::collection())?;

            create_collection::<D, LocalDeviceRecord>(&tx)?;
            create_collection::<D, PreKeyRecord>(&tx)?;
            create_collection::<D, SenderKeyRecord>(&tx)?;
            create_collection::<D, SessionRecord>(&tx)?;
            create_collection::<D, SignedPreKeyRecord>(&tx)?;
            create_collection::<D, KyberPreKeyRecord>(&tx)?;

            tx.delete_collection(UserInfoRecord::collection())?;
            tx.delete_collection(UserProfileRecord::collection())?;
            tx.delete_collection(DraftsRecord::collection())?;
            tx.delete_collection(UserDeviceRecord::collection())?;

            create_collection::<D, UserInfoRecord>(&tx)?;
            create_collection::<D, UserProfileRecord>(&tx)?;
            create_collection::<D, DraftsRecord>(&tx)?;
            create_collection::<D, UserDeviceRecord>(&tx)?;
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
        let user_device_repo = Arc::new(CachingUserDeviceRepository::new(
            d.store.clone(),
            d.xmpp.clone(),
        ));
        let user_info_repo = Arc::new(CachingUserInfoRepository::new(
            d.store.clone(),
            d.xmpp.clone(),
        ));
        let user_profile_repo = Arc::new(CachingUserProfileRepository::new(
            d.store.clone(),
            d.xmpp.clone(),
        ));
        let local_room_settings_repo = Arc::new(LocalRoomSettingsRepository::new(d.store.clone()));

        let message_migration_domain_service_dependencies =
            MessageMigrationDomainServiceDependencies {
                message_archive_service: d.xmpp.clone(),
                messaging_service: d.xmpp.clone(),
            };

        let message_migration_domain_service = Arc::new(MessageMigrationDomainService::from(
            message_migration_domain_service_dependencies,
        ));

        let encryption_domain_service_dependencies = EncryptionDomainServiceDependencies {
            ctx: ctx.clone(),
            encryption_keys_repo: Arc::new(EncryptionKeysRepository::new(d.store.clone())),
            encryption_service: d.encryption_service,
            message_repo: messages_repo.clone(),
            messaging_service: d.xmpp.clone(),
            rng_provider: d.rng_provider.clone(),
            session_repo: Arc::new(SessionRepository::new(d.store.clone())),
            time_provider: time_provider.clone(),
            user_device_id_provider: d.user_device_id_provider,
            user_device_repo: user_device_repo.clone(),
            user_device_service: d.xmpp.clone(),
        };
        let encryption_domain_service = Arc::new(EncryptionDomainService::from(
            encryption_domain_service_dependencies,
        ));

        let message_archive_domain_service_dependencies = MessageArchiveDomainServiceDependencies {
            ctx: ctx.clone(),
            encryption_domain_service: encryption_domain_service.clone(),
            local_room_settings_repo: local_room_settings_repo.clone(),
            message_archive_service: d.xmpp.clone(),
            message_repo: messages_repo.clone(),
            time_provider: time_provider.clone(),
        };

        let message_archive_domain_service = Arc::new(MessageArchiveDomainService::from(
            message_archive_domain_service_dependencies,
        ));

        let rooms_domain_service_dependencies = RoomsDomainServiceDependencies {
            account_settings_repo: account_settings_repo.clone(),
            client_event_dispatcher: client_event_dispatcher.clone(),
            connected_rooms_repo: connected_rooms_repo.clone(),
            ctx: ctx.clone(),
            encryption_domain_service: encryption_domain_service.clone(),
            id_provider: d.short_id_provider.clone(),
            message_migration_domain_service: message_migration_domain_service.clone(),
            room_attributes_service: d.xmpp.clone(),
            room_management_service: d.xmpp.clone(),
            room_participation_service: d.xmpp.clone(),
            synced_room_settings_service: d.xmpp.clone(),
            user_info_repo: user_info_repo.clone(),
            user_profile_repo: user_profile_repo.clone(),
            message_archive_domain_service: message_archive_domain_service.clone(),
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
            ctx: ctx.clone(),
            client_event_dispatcher: client_event_dispatcher.clone(),
            contact_list_repo: Arc::new(CachingContactsRepository::new(d.xmpp.clone())),
            contact_list_service: d.xmpp.clone(),
            presence_sub_requests_repo: Arc::new(PresenceSubRequestsRepository::new()),
        };

        let contact_list_domain_service = Arc::new(ContactListDomainService::from(
            contact_list_domain_service_dependencies,
        ));

        let block_list_domain_service_dependencies = BlockListDomainServiceDependencies {
            ctx: ctx.clone(),
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
            let drafts_repo = drafts_repo.clone();
            let encryption_domain_service = encryption_domain_service.clone();
            let id_provider = id_provider.clone();
            let message_repo = messages_repo.clone();
            let sidebar_domain_service = sidebar_domain_service.clone();
            let time_provider = time_provider.clone();
            let user_profile_repo = user_profile_repo.clone();
            let xmpp = d.xmpp.clone();

            RoomFactory::new(Arc::new(move |data| {
                RoomInner {
                    attributes_service: xmpp.clone(),
                    client_event_dispatcher: client_event_dispatcher.clone(),
                    ctx: ctx.clone(),
                    data: data.clone(),
                    drafts_repo: drafts_repo.clone(),
                    encryption_domain_service: encryption_domain_service.clone(),
                    id_provider: id_provider.clone(),
                    message_archive_service: xmpp.clone(),
                    message_repo: message_repo.clone(),
                    messaging_service: xmpp.clone(),
                    participation_service: xmpp.clone(),
                    synced_room_settings_service: xmpp.clone(),
                    sidebar_domain_service: sidebar_domain_service.clone(),
                    time_provider: time_provider.clone(),
                    user_profile_repo: user_profile_repo.clone(),
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
            encryption_domain_service,
            id_provider,
            local_room_settings_repo,
            message_archive_service: d.xmpp.clone(),
            messages_repo,
            messaging_service: d.xmpp.clone(),
            offline_messages_repo: Arc::new(OfflineMessagesRepository::new()),
            request_handling_service: d.xmpp.clone(),
            rng_provider: d.rng_provider.clone(),
            room_attributes_service: d.xmpp.clone(),
            room_factory,
            room_management_service: d.xmpp.clone(),
            room_participation_service: d.xmpp.clone(),
            rooms_domain_service,
            server_event_handler_queue: d.server_event_handler_queue,
            short_id_provider: d.short_id_provider,
            sidebar_domain_service,
            time_provider,
            upload_service: d.xmpp.clone(),
            user_account_service: d.xmpp.clone(),
            user_device_repo,
            user_info_repo,
            user_info_service: d.xmpp.clone(),
            user_profile_repo,
            user_profile_service: d.xmpp.clone(),
        }
    }
}
