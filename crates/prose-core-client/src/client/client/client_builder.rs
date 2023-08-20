// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use prose_xmpp::client::ConnectorProvider;
use prose_xmpp::mods::{Caps, Chat, Profile, Roster, Status, MAM};
use prose_xmpp::{
    ns, Client as XMPPClient, ClientBuilder as XMPPClientBuilder, IDProvider, SystemTimeProvider,
    TimeProvider,
};

use crate::avatar_cache::AvatarCache;
use crate::client::client::client::ClientInner;
use crate::data_cache::DataCache;
use crate::types::{Capabilities, Feature, SoftwareVersion};
use crate::{Client, ClientDelegate};

pub struct UndefinedDataCache {}
pub struct UndefinedAvatarCache {}

pub struct ClientBuilder<D, A> {
    builder: XMPPClientBuilder,
    data_cache: D,
    avatar_cache: A,
    time_provider: Arc<dyn TimeProvider>,
    software_version: SoftwareVersion,
    delegate: Option<Box<dyn ClientDelegate<D, A>>>,
}

impl ClientBuilder<UndefinedDataCache, UndefinedAvatarCache> {
    pub fn new() -> Self {
        ClientBuilder {
            builder: XMPPClient::builder(),
            data_cache: UndefinedDataCache {},
            avatar_cache: UndefinedAvatarCache {},
            time_provider: Arc::new(SystemTimeProvider::default()),
            software_version: SoftwareVersion::default(),
            delegate: None,
        }
    }
}

impl<A> ClientBuilder<UndefinedDataCache, A> {
    pub fn set_data_cache<D2: DataCache>(self, data_cache: D2) -> ClientBuilder<D2, A> {
        ClientBuilder {
            builder: self.builder,
            data_cache,
            avatar_cache: self.avatar_cache,
            time_provider: self.time_provider,
            software_version: self.software_version,
            delegate: None,
        }
    }
}

impl<D> ClientBuilder<D, UndefinedAvatarCache> {
    pub fn set_avatar_cache<A2: AvatarCache>(self, avatar_cache: A2) -> ClientBuilder<D, A2> {
        ClientBuilder {
            builder: self.builder,
            data_cache: self.data_cache,
            avatar_cache,
            time_provider: self.time_provider,
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
}

impl<D: DataCache, A: AvatarCache> ClientBuilder<D, A> {
    pub fn set_delegate(mut self, delegate: Option<Box<dyn ClientDelegate<D, A>>>) -> Self {
        self.delegate = delegate;
        self
    }

    pub fn build(self) -> Client<D, A> {
        let caps = Capabilities::new(
            self.software_version.name.clone(),
            "https://prose.org",
            vec![
                Feature::Name(ns::JABBER_CLIENT),
                Feature::Name(ns::AVATAR_DATA),
                Feature::Name(ns::AVATAR_METADATA),
                Feature::Name(ns::CHATSTATES),
                Feature::Name(ns::DISCO_INFO),
                Feature::Name(ns::RSM),
                Feature::Name(ns::CAPS),
                Feature::Name(ns::PING),
                Feature::Name(ns::PUBSUB),
                Feature::Name(ns::PUBSUB_EVENT),
                Feature::Name(ns::ROSTER),
                Feature::Name(ns::REACTIONS),
                Feature::Name(ns::RECEIPTS),
                Feature::Name(ns::CHAT_MARKERS),
                Feature::Name(ns::MESSAGE_CORRECT),
                Feature::Name(ns::RETRACT),
                Feature::Name(ns::FASTEN),
                Feature::Name(ns::DELAY),
                Feature::Name(ns::FALLBACK),
                Feature::Name(ns::HINTS),
                Feature::Name(ns::MAM),
                Feature::Name(ns::TIME),
                Feature::Name(ns::VERSION),
                Feature::Name(ns::LAST_ACTIVITY),
                Feature::Name(ns::USER_ACTIVITY),
                Feature::Name(ns::VCARD4),
                Feature::Notify(ns::PUBSUB),
                Feature::Notify(ns::USER_ACTIVITY),
                Feature::Notify(ns::AVATAR_METADATA),
                Feature::Notify(ns::VCARD4),
                Feature::Notify(ns::BOOKMARKS2),
            ],
        );

        let inner = Arc::new(ClientInner {
            caps,
            data_cache: self.data_cache,
            avatar_cache: self.avatar_cache,
            time_provider: self.time_provider.clone(),
            software_version: self.software_version,
            delegate: self.delegate,
            presences: Default::default(),
        });

        let event_inner = inner.clone();

        let client = self
            .builder
            .add_mod(Caps::default())
            .add_mod(MAM::default())
            .add_mod(Chat::default())
            .add_mod(Profile::default())
            .add_mod(Roster::default())
            .add_mod(Status::default())
            .set_time_provider(self.time_provider)
            .set_event_handler(Box::new(move |xmpp_client, event| {
                let client = Client {
                    client: xmpp_client,
                    inner: event_inner.clone(),
                };
                async move { client.handle_event(event).await }
            }))
            .build();

        Client { client, inner }
    }
}
