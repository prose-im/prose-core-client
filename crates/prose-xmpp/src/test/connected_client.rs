// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use jid::{BareJid, FullJid};
use parking_lot::RwLock;

use crate::test::{BareJidTestAdditions, Connection, Connector, IncrementingIDProvider};
use crate::{Client, Event, IDProvider};

#[async_trait(?Send)]
pub trait ClientTestAdditions {
    async fn connected_client() -> Result<ConnectedClient>;
    async fn connected_client_with_current_user(jid: FullJid) -> Result<ConnectedClient>;
}

pub struct ConnectedClient {
    pub client: Client,
    pub connection: Connection,
    pub id_provider: Arc<IncrementingIDProvider>,
    pub sent_events: Arc<RwLock<Vec<Event>>>,
}

#[async_trait(?Send)]
impl ClientTestAdditions for Client {
    async fn connected_client() -> Result<ConnectedClient> {
        Self::connected_client_with_current_user(
            FullJid::from_str(&format!("{}/test", BareJid::ours())).unwrap(),
        )
        .await
    }

    async fn connected_client_with_current_user(jid: FullJid) -> Result<ConnectedClient> {
        let connection = Connection::default();
        let id_provider = Arc::new(IncrementingIDProvider::new("id"));
        let sent_events = Arc::new(RwLock::new(vec![]));

        let handler_events = sent_events.clone();
        let client = Client::builder()
            .set_connector_provider(Connector::provider(connection.clone()))
            .set_id_provider(id_provider.clone() as Arc<dyn IDProvider>)
            .set_event_handler(Box::new(move |_, event| {
                handler_events.write().push(event);
                async {}
            }))
            .build();

        client.connect(&jid, "".into()).await?;

        id_provider.reset();
        sent_events.write().clear();

        Ok(ConnectedClient {
            client,
            connection,
            id_provider,
            sent_events,
        })
    }
}

impl ConnectedClient {
    pub fn sent_events(&self) -> Vec<Event> {
        self.sent_events.read().clone()
    }
}
