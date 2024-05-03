// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use prose_store::prelude::{PlatformDriver, Store};
use prose_xmpp::client::ConnectorProvider;
use prose_xmpp::{ns, IDProvider, SystemTimeProvider, TimeProvider, UUIDProvider};

use crate::app::deps::{
    AppContext, AppDependencies, DynEncryptionService, DynIDProvider, DynRngProvider,
    DynTimeProvider, DynUserDeviceIdProvider,
};
use crate::app::event_handlers::{
    BlockListEventHandler, BookmarksEventHandler, ConnectionEventHandler, ContactListEventHandler,
    MessagesEventHandler, RequestsEventHandler, RoomsEventHandler, ServerEventHandlerQueue,
    UserDevicesEventHandler, UserStateEventHandler,
};
use crate::app::services::{
    AccountService, ConnectionService, ContactListService, RoomsService, UserDataService,
};
use crate::client::ClientInner;
use crate::domain::encryption::services::{RandUserDeviceIdProvider, UserDeviceIdProvider};
use crate::domain::general::models::{Capabilities, Feature, SoftwareVersion};
use crate::infra::avatars::AvatarCache;
use crate::infra::general::{NanoIDProvider, OsRngProvider, RngProvider};
use crate::infra::platform_dependencies::PlatformDependencies;
use crate::infra::xmpp::{XMPPClient, XMPPClientBuilder};
use crate::services::{BlockListService, CacheService, SidebarService, UploadService};
use crate::{Client, ClientDelegate};

pub struct UndefinedStore;
pub struct UndefinedAvatarCache;
pub struct UndefinedEncryptionService;

pub struct ClientBuilder<S, A, E> {
    avatar_cache: A,
    builder: XMPPClientBuilder,
    delegate: Option<Box<dyn ClientDelegate>>,
    encryption_service: E,
    id_provider: DynIDProvider,
    rng_provider: DynRngProvider,
    short_id_provider: DynIDProvider,
    software_version: SoftwareVersion,
    store: S,
    time_provider: DynTimeProvider,
    user_device_id_provider: DynUserDeviceIdProvider,
}

impl ClientBuilder<UndefinedStore, UndefinedAvatarCache, UndefinedEncryptionService> {
    pub(crate) fn new() -> Self {
        ClientBuilder {
            avatar_cache: UndefinedAvatarCache {},
            builder: XMPPClient::builder(),
            delegate: None,
            encryption_service: UndefinedEncryptionService,
            id_provider: Arc::new(UUIDProvider::default()),
            rng_provider: Arc::new(OsRngProvider),
            short_id_provider: Arc::new(NanoIDProvider::default()),
            software_version: SoftwareVersion::default(),
            store: UndefinedStore,
            time_provider: Arc::new(SystemTimeProvider::default()),
            user_device_id_provider: Arc::new(RandUserDeviceIdProvider::default()),
        }
    }
}

impl<A, E> ClientBuilder<UndefinedStore, A, E> {
    pub fn set_store(
        self,
        store: Store<PlatformDriver>,
    ) -> ClientBuilder<Store<PlatformDriver>, A, E> {
        ClientBuilder {
            avatar_cache: self.avatar_cache,
            builder: self.builder,
            delegate: None,
            encryption_service: self.encryption_service,
            id_provider: self.id_provider,
            rng_provider: self.rng_provider,
            short_id_provider: self.short_id_provider,
            software_version: self.software_version,
            store,
            time_provider: self.time_provider,
            user_device_id_provider: self.user_device_id_provider,
        }
    }
}

impl<D, E> ClientBuilder<D, UndefinedAvatarCache, E> {
    pub fn set_avatar_cache<A2: AvatarCache>(self, avatar_cache: A2) -> ClientBuilder<D, A2, E> {
        ClientBuilder {
            avatar_cache,
            builder: self.builder,
            delegate: None,
            encryption_service: self.encryption_service,
            id_provider: self.id_provider,
            rng_provider: self.rng_provider,
            short_id_provider: self.short_id_provider,
            software_version: self.software_version,
            store: self.store,
            time_provider: self.time_provider,
            user_device_id_provider: self.user_device_id_provider,
        }
    }
}

impl<S, A> ClientBuilder<S, A, UndefinedEncryptionService> {
    pub fn set_encryption_service(
        self,
        encryption_service: DynEncryptionService,
    ) -> ClientBuilder<S, A, DynEncryptionService> {
        ClientBuilder {
            avatar_cache: self.avatar_cache,
            builder: self.builder,
            delegate: None,
            encryption_service,
            id_provider: self.id_provider,
            rng_provider: self.rng_provider,
            short_id_provider: self.short_id_provider,
            software_version: self.software_version,
            store: self.store,
            time_provider: self.time_provider,
            user_device_id_provider: self.user_device_id_provider,
        }
    }
}

impl<D, A, E> ClientBuilder<D, A, E> {
    pub fn set_connector_provider(mut self, connector_provider: ConnectorProvider) -> Self {
        self.builder = self.builder.set_connector_provider(connector_provider);
        self
    }

    pub fn set_id_provider<P: IDProvider + 'static>(mut self, id_provider: P) -> Self {
        let id_provider: DynIDProvider = Arc::new(id_provider);
        self.builder = self.builder.set_id_provider(id_provider.clone());
        self.id_provider = id_provider;
        self
    }

    pub fn set_short_id_provider<P: IDProvider + 'static>(mut self, id_provider: P) -> Self {
        self.short_id_provider = Arc::new(id_provider);
        self
    }

    pub fn set_rng_provider<P: RngProvider + 'static>(mut self, rng_provider: P) -> Self {
        self.rng_provider = Arc::new(rng_provider);
        self
    }

    pub fn set_user_device_id_provider<P: UserDeviceIdProvider + 'static>(
        mut self,
        id_provider: P,
    ) -> Self {
        self.user_device_id_provider = Arc::new(id_provider);
        self
    }

    pub fn set_time_provider<T: TimeProvider + 'static>(mut self, time_provider: T) -> Self {
        self.time_provider = Arc::new(time_provider);
        self
    }

    pub fn set_software_version(mut self, software_version: SoftwareVersion) -> Self {
        self.software_version = software_version;
        self
    }

    pub fn set_delegate(mut self, delegate: Option<Box<dyn ClientDelegate>>) -> Self {
        self.delegate = delegate;
        self
    }
}

impl<A: AvatarCache + 'static> ClientBuilder<Store<PlatformDriver>, A, DynEncryptionService> {
    pub fn build(self) -> Client {
        let capabilities = Capabilities::new(
            self.software_version.name.clone(),
            "https://prose.org",
            vec![
                Feature::Name(ns::AVATAR_DATA),
                Feature::Name(ns::AVATAR_METADATA),
                Feature::Name(ns::BOOKMARKS2),
                Feature::Name(ns::CAPS),
                Feature::Name(ns::CHATSTATES),
                Feature::Name(ns::CHAT_MARKERS),
                Feature::Name(ns::DELAY),
                Feature::Name(ns::DISCO_INFO),
                Feature::Name(ns::FALLBACK),
                Feature::Name(ns::FASTEN),
                Feature::Name(ns::HINTS),
                Feature::Name(ns::JABBER_CLIENT),
                Feature::Name(ns::LAST_ACTIVITY),
                Feature::Name(ns::MAM),
                Feature::Name(ns::MESSAGE_CORRECT),
                Feature::Name(ns::OUT_OF_BAND_DATA),
                Feature::Name(ns::PING),
                Feature::Name(ns::PUBSUB),
                Feature::Name(ns::PUBSUB_EVENT),
                Feature::Name(ns::REACTIONS),
                Feature::Name(ns::RECEIPTS),
                Feature::Name(ns::REFERENCE),
                Feature::Name(ns::RETRACT),
                Feature::Name(ns::ROSTER),
                Feature::Name(ns::RSM),
                Feature::Name(ns::TIME),
                Feature::Name(ns::USER_ACTIVITY),
                Feature::Name(ns::VCARD4),
                Feature::Name(ns::VERSION),
                Feature::Notify(crate::infra::xmpp::type_conversions::bookmark::ns::PROSE_BOOKMARK),
                Feature::Notify(ns::AVATAR_METADATA),
                Feature::Notify(ns::BOOKMARKS),
                Feature::Notify(ns::BOOKMARKS2),
                Feature::Notify(ns::LEGACY_OMEMO_DEVICELIST),
                Feature::Notify(ns::PUBSUB),
                Feature::Notify(ns::USER_ACTIVITY),
                Feature::Notify(ns::VCARD4),
            ],
        );

        let handler_queue = Arc::new(ServerEventHandlerQueue::new());

        let xmpp_client = Arc::new(
            {
                let handler_queue = handler_queue.clone();
                self.builder.set_event_handler(move |_, event| {
                    let handler_queue = handler_queue.clone();
                    async move { handler_queue.handle_event(event).await }
                })
            }
            .build(),
        );

        #[cfg(feature = "test")]
        let event_dispatcher = Arc::new(crate::infra::events::ImmediateClientEventDispatcher::new(
            self.delegate,
        ));
        #[cfg(not(feature = "test"))]
        let event_dispatcher = Arc::new(
            crate::infra::events::CoalescingClientEventDispatcher::new(self.delegate),
        );

        let dependencies: AppDependencies = PlatformDependencies {
            ctx: AppContext::new(capabilities, self.software_version),
            encryption_service: self.encryption_service,
            id_provider: self.id_provider,
            rng_provider: self.rng_provider,
            short_id_provider: self.short_id_provider,
            store: self.store,
            time_provider: self.time_provider,
            user_device_id_provider: self.user_device_id_provider,
            xmpp: xmpp_client.clone(),
            avatar_cache: Box::new(self.avatar_cache),
            client_event_dispatcher: event_dispatcher.clone(),
        }
        .into();

        handler_queue.set_handlers(vec![
            Box::new(ConnectionEventHandler::from(&dependencies)),
            Box::new(RequestsEventHandler::from(&dependencies)),
            Box::new(UserStateEventHandler::from(&dependencies)),
            Box::new(MessagesEventHandler::from(&dependencies)),
            Box::new(RoomsEventHandler::from(&dependencies)),
            Box::new(BookmarksEventHandler::from(&dependencies)),
            Box::new(ContactListEventHandler::from(&dependencies)),
            Box::new(BlockListEventHandler::from(&dependencies)),
            Box::new(UserDevicesEventHandler::from(&dependencies)),
        ]);

        let client_inner = Arc::new(ClientInner {
            connection: ConnectionService::from(&dependencies),
            account: AccountService::from(&dependencies),
            contact_list: ContactListService::from(&dependencies),
            ctx: dependencies.ctx.clone(),
            #[cfg(feature = "debug")]
            debug: crate::services::DebugService::new(xmpp_client.clone()),
            rooms: RoomsService::from(&dependencies),
            sidebar: SidebarService::from(&dependencies),
            uploads: UploadService::from(&dependencies),
            user_data: UserDataService::from(&dependencies),
            cache: CacheService::from(&dependencies),
            block_list: BlockListService::from(&dependencies),
        });

        event_dispatcher.set_client_inner(Arc::downgrade(&client_inner));
        event_dispatcher.set_room_factory(dependencies.room_factory);

        Client::from(client_inner)
    }
}
