// prose-core-client
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
use crate::types::{Capabilities, Feature};
use crate::{Client, ClientDelegate};

pub struct UndefinedDataCache {}
pub struct UndefinedAvatarCache {}

pub struct ClientBuilder<D, A> {
    builder: XMPPClientBuilder,
    data_cache: D,
    avatar_cache: A,
    time_provider: Arc<dyn TimeProvider>,
    delegate: Option<Box<dyn ClientDelegate<D, A>>>,
}

impl ClientBuilder<UndefinedDataCache, UndefinedAvatarCache> {
    pub fn new() -> Self {
        ClientBuilder {
            builder: XMPPClient::builder(),
            data_cache: UndefinedDataCache {},
            avatar_cache: UndefinedAvatarCache {},
            time_provider: Arc::new(SystemTimeProvider::default()),
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
}

impl<D: DataCache, A: AvatarCache> ClientBuilder<D, A> {
    pub fn set_delegate(mut self, delegate: Option<Box<dyn ClientDelegate<D, A>>>) -> Self {
        self.delegate = delegate;
        self
    }

    pub fn build(self) -> Client<D, A> {
        let caps = Capabilities::new(
            "Prose",
            "https://www.prose.org",
            vec![
                Feature::new(ns::AVATAR_DATA, false),
                Feature::new(ns::AVATAR_METADATA, false),
                Feature::new(ns::AVATAR_METADATA, true),
                Feature::new(ns::CHATSTATES, false),
                Feature::new(ns::PING, false),
                Feature::new(ns::PUBSUB, false),
                Feature::new(ns::PUBSUB, true),
                Feature::new(ns::RECEIPTS, false),
                Feature::new(ns::VCARD4, false),
                Feature::new(ns::VCARD4, true),
                Feature::new(ns::USER_ACTIVITY, false),
                Feature::new(ns::USER_ACTIVITY, true),
                Feature::new(ns::TIME, false),
            ],
        );

        let inner = Arc::new(ClientInner {
            caps,
            data_cache: self.data_cache,
            avatar_cache: self.avatar_cache,
            time_provider: self.time_provider.clone(),
            delegate: self.delegate,
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
