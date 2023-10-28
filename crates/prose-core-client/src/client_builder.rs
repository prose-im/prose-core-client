// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;

use prose_store::prelude::PlatformDriver;
use prose_xmpp::client::ConnectorProvider;
use prose_xmpp::{ns, IDProvider, SystemTimeProvider, TimeProvider, UUIDProvider};

use crate::app::deps::{AppContext, AppDependencies, AppServiceDependencies};
use crate::app::event_handlers::{
    ClientEventDispatcher, ConnectionEventHandler, RequestsEventHandler, UserStateEventHandler,
    XMPPEventHandlerQueue,
};
use crate::app::services::{AccountService, ContactsService, RoomsService, UserDataService};
use crate::client::ClientInner;
use crate::domain::general::models::{Capabilities, Feature, SoftwareVersion};
use crate::infra::avatars::AvatarCache;
use crate::infra::platform_dependencies::{open_store, PlatformDependencies};
use crate::infra::xmpp::{XMPPClient, XMPPClientBuilder};
use crate::{Client, ClientDelegate};

pub struct UndefinedDriver {}
pub struct UndefinedAvatarCache {}

pub struct ClientBuilder<D, A> {
    builder: XMPPClientBuilder,
    driver: D,
    avatar_cache: A,
    time_provider: Arc<dyn TimeProvider>,
    id_provider: Arc<dyn IDProvider>,
    software_version: SoftwareVersion,
    delegate: Option<Box<dyn ClientDelegate>>,
}

impl ClientBuilder<UndefinedDriver, UndefinedAvatarCache> {
    pub fn new() -> Self {
        ClientBuilder {
            builder: XMPPClient::builder(),
            driver: UndefinedDriver {},
            avatar_cache: UndefinedAvatarCache {},
            time_provider: Arc::new(SystemTimeProvider::default()),
            id_provider: Arc::new(UUIDProvider::default()),
            software_version: SoftwareVersion::default(),
            delegate: None,
        }
    }
}

impl<A> ClientBuilder<UndefinedDriver, A> {
    pub fn set_driver(self, driver: PlatformDriver) -> ClientBuilder<PlatformDriver, A> {
        ClientBuilder {
            builder: self.builder,
            driver,
            avatar_cache: self.avatar_cache,
            time_provider: self.time_provider,
            id_provider: self.id_provider,
            software_version: self.software_version,
            delegate: None,
        }
    }
}

impl<D> ClientBuilder<D, UndefinedAvatarCache> {
    pub fn set_avatar_cache<A2: AvatarCache>(self, avatar_cache: A2) -> ClientBuilder<D, A2> {
        ClientBuilder {
            builder: self.builder,
            driver: self.driver,
            avatar_cache,
            time_provider: self.time_provider,
            id_provider: self.id_provider,
            software_version: self.software_version,
            delegate: None,
        }
    }
}

impl<D, A> ClientBuilder<D, A> {
    pub fn set_connector_provider(mut self, connector_provider: ConnectorProvider) -> Self {
        self.builder = self.builder.set_connector_provider(connector_provider);
        self
    }

    pub fn set_id_provider<P: IDProvider + 'static>(mut self, id_provider: P) -> Self {
        self.builder = self.builder.set_id_provider(id_provider);
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

impl<A: AvatarCache + 'static> ClientBuilder<PlatformDriver, A> {
    pub async fn build(self) -> Result<Client> {
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
                Feature::Name(ns::PING),
                Feature::Name(ns::PUBSUB),
                Feature::Name(ns::PUBSUB_EVENT),
                Feature::Name(ns::REACTIONS),
                Feature::Name(ns::RECEIPTS),
                Feature::Name(ns::RETRACT),
                Feature::Name(ns::ROSTER),
                Feature::Name(ns::RSM),
                Feature::Name(ns::TIME),
                Feature::Name(ns::USER_ACTIVITY),
                Feature::Name(ns::VCARD4),
                Feature::Name(ns::VERSION),
                Feature::Notify(ns::AVATAR_METADATA),
                Feature::Notify(ns::BOOKMARKS),
                Feature::Notify(ns::BOOKMARKS2),
                Feature::Notify(ns::PUBSUB),
                Feature::Notify(ns::USER_ACTIVITY),
                Feature::Notify(ns::VCARD4),
            ],
        );

        let handler_queue = Arc::new(XMPPEventHandlerQueue::new());

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

        let event_dispatcher = Arc::new(ClientEventDispatcher::new(self.delegate));

        let app_service_deps = AppServiceDependencies {
            time_provider: self.time_provider,
            id_provider: self.id_provider,
            event_dispatcher: event_dispatcher.clone(),
        };

        let dependencies: AppDependencies = PlatformDependencies {
            ctx: AppContext::new(capabilities, self.software_version),
            app_service_deps,
            store: open_store(self.driver).await?,
            xmpp: xmpp_client.clone(),
            avatar_cache: Box::new(self.avatar_cache),
        }
        .into();

        handler_queue.set_handlers(vec![
            Box::new(ConnectionEventHandler::from(&dependencies)),
            Box::new(RequestsEventHandler::from(&dependencies)),
            Box::new(UserStateEventHandler::from(&dependencies)),
            // Box::new(RoomsEventHandler::from(&dependencies)),
        ]);

        let client_inner = Arc::new(ClientInner {
            account: AccountService::from(&dependencies),
            contacts: ContactsService::from(&dependencies),
            rooms: RoomsService::from(&dependencies),
            user_data: UserDataService::from(&dependencies),
        });

        event_dispatcher.set_client_inner(Arc::downgrade(&client_inner));

        Ok(Client::from(client_inner))
    }
}
