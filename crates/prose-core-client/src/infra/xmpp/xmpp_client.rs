// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;
use std::sync::Arc;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use prose_xmpp::client::ConnectorProvider;
use prose_xmpp::{mods, Client, ClientBuilder, Event, IDProvider, TimeProvider};

#[derive(Clone)]
pub struct XMPPClient {
    pub(crate) client: Arc<Client>,
}

impl XMPPClient {
    pub fn builder() -> XMPPClientBuilder {
        XMPPClientBuilder {
            builder: Client::builder(),
        }
    }
}

pub struct XMPPClientBuilder {
    builder: ClientBuilder,
}

impl XMPPClientBuilder {
    pub fn set_connector_provider(mut self, connector_provider: ConnectorProvider) -> Self {
        self.builder = self.builder.set_connector_provider(connector_provider);
        self
    }

    pub fn set_id_provider<P: IDProvider + 'static>(mut self, id_provider: P) -> Self {
        self.builder = self.builder.set_id_provider(id_provider);
        self
    }

    pub fn set_time_provider<T: TimeProvider + 'static>(mut self, time_provider: T) -> Self {
        self.builder = self.builder.set_time_provider(time_provider);
        self
    }

    pub fn set_event_handler<T>(
        mut self,
        handler: impl Fn(Client, Event) -> T + SendUnlessWasm + SyncUnlessWasm + 'static,
    ) -> Self
    where
        T: Future<Output = ()> + SendUnlessWasm + 'static,
    {
        self.builder = self.builder.set_event_handler(handler);
        self
    }

    pub fn build(self) -> XMPPClient {
        let client = self
            .builder
            .add_mod(mods::Bookmark2::default())
            .add_mod(mods::Bookmark::default())
            .add_mod(mods::Caps::default())
            .add_mod(mods::Chat::default())
            .add_mod(mods::MAM::default())
            .add_mod(mods::MUC::default())
            .add_mod(mods::Profile::default())
            .add_mod(mods::Roster::default())
            .add_mod(mods::Status::default())
            .build();

        XMPPClient {
            client: Arc::new(client),
        }
    }
}
