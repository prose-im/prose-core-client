// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use prose_xmpp::client::ConnectorProvider;
use prose_xmpp::mods::{Bookmark, Bookmark2, Caps, Chat, Profile, Roster, Status, MAM, MUC};
use prose_xmpp::{
    ns, Client as XMPPClient, ClientBuilder as XMPPClientBuilder, IDProvider, SystemTimeProvider,
    TimeProvider, UUIDProvider,
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
    id_provider: Arc<dyn IDProvider>,
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
            id_provider: Arc::new(UUIDProvider::default()),
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
            data_cache: self.data_cache,
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

        let inner = Arc::new(ClientInner {
            caps,
            data_cache: self.data_cache,
            avatar_cache: self.avatar_cache,
            id_provider: self.id_provider.clone(),
            time_provider: self.time_provider.clone(),
            software_version: self.software_version,
            delegate: self.delegate,
            presences: Default::default(),
            muc_service: Default::default(),
            bookmarks: Default::default(),
            connected_rooms: Default::default(),
        });

        let event_inner = inner.clone();

        let client = self
            .builder
            .add_mod(Bookmark2::default())
            .add_mod(Bookmark::default())
            .add_mod(Caps::default())
            .add_mod(Chat::default())
            .add_mod(MAM::default())
            .add_mod(MUC::default())
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
