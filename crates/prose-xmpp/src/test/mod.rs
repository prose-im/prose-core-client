// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{mods, Client, Event, IDProvider};
use anyhow::Result;
use async_trait::async_trait;
#[cfg(feature = "test")]
pub use connector::{Connection, Connector};
pub use incrementing_id_provider::IncrementingIDProvider;
use jid::{BareJid, DomainPart, FullJid, NodePart};
use parking_lot::RwLock;
use std::str::FromStr;
use std::sync::Arc;

mod connector;
mod incrementing_id_provider;

pub trait StrExt {
    fn to_xml_result_string(&self) -> String;
}

impl<T> StrExt for T
where
    T: AsRef<str>,
{
    fn to_xml_result_string(&self) -> String {
        let mut result = self.as_ref().to_string();
        result.retain(|c| c != '\n' && c != '\t');
        result.replace("  ", "")
    }
}

#[macro_export]
macro_rules! jid_str {
    ($jid:expr) => {
        $jid.parse::<jid::Jid>().unwrap()
    };
}

pub trait BareJidTestAdditions {
    fn ours() -> BareJid;
    fn theirs() -> BareJid;
}

impl BareJidTestAdditions for BareJid {
    fn ours() -> BareJid {
        BareJid::from_parts(
            Some(&NodePart::new("test").unwrap()),
            &DomainPart::new("prose.org").unwrap(),
        )
    }

    fn theirs() -> BareJid {
        BareJid::from_parts(
            Some(&NodePart::new("friend").unwrap()),
            &DomainPart::new("prose.org").unwrap(),
        )
    }
}

#[async_trait(?Send)]
pub trait ClientTestAdditions {
    async fn connected_client() -> Result<ConnectedClient>;
}

pub struct ConnectedClient {
    pub client: Client,
    pub connection: Arc<Connection>,
    pub id_provider: Arc<IncrementingIDProvider>,
    pub sent_events: Arc<RwLock<Vec<Event>>>,
}

#[async_trait(?Send)]
impl ClientTestAdditions for Client {
    async fn connected_client() -> Result<ConnectedClient> {
        let connection = Arc::new(Connection::default());
        let id_provider = Arc::new(IncrementingIDProvider::new());
        let sent_events = Arc::new(RwLock::new(vec![]));

        let handler_events = sent_events.clone();
        let client = Client::builder()
            .set_connector_provider(Connector::provider(connection.clone()))
            .set_id_provider(id_provider.clone() as Arc<dyn IDProvider>)
            .set_event_handler(Box::new(move |_, event| {
                handler_events.write().push(event);
                async {}
            }))
            .add_mod(mods::Bookmark::default())
            .add_mod(mods::Bookmark2::default())
            .add_mod(mods::Caps::default())
            .add_mod(mods::Chat::default())
            .add_mod(mods::MAM::default())
            .add_mod(mods::Profile::default())
            .add_mod(mods::Roster::default())
            .add_mod(mods::Status::default())
            .build();

        client
            .connect(
                &FullJid::from_str(&format!("{}/test", BareJid::ours()))?,
                "",
            )
            .await?;

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
