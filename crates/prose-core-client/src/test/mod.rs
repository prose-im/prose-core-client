// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, FixedOffset, TimeZone, Utc};
use jid::{BareJid, DomainPart, FullJid, NodePart};
use std::str::FromStr;
use std::sync::Arc;

pub use constant_time_provider::ConstantTimeProvider;
pub use incrementing_id_provider::IncrementingIDProvider;
pub use message_builder::MessageBuilder;
use prose_xmpp::{test, IDProvider, SystemTimeProvider, TimeProvider};

use crate::types::{Availability, SoftwareVersion};
use crate::{
    avatar_cache::NoopAvatarCache, data_cache::sqlite::SQLiteCache, Client, ClientBuilder,
};

mod constant_time_provider;
mod incrementing_id_provider;
mod message_builder;

pub trait BareJidTestAdditions {
    fn ours() -> BareJid;
    fn theirs() -> BareJid;
}

pub trait DateTimeTestAdditions {
    fn test_timestamp() -> DateTime<FixedOffset>;
    fn test_timestamp_adding(seconds: u32) -> DateTime<FixedOffset>;
}

#[macro_export]
macro_rules! jid_str {
    ($jid:expr) => {
        $jid.parse::<jid::Jid>().unwrap()
    };
}

#[async_trait(?Send)]
pub trait ClientTestAdditions {
    async fn connected_client() -> Result<ConnectedClient>;
    async fn connected_client_with_time_provider<T: TimeProvider + 'static>(
        time_provider: T,
    ) -> Result<ConnectedClient>;
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

impl DateTimeTestAdditions for Utc {
    fn test_timestamp() -> DateTime<FixedOffset> {
        Utc.with_ymd_and_hms(2023, 06, 02, 17, 00, 00)
            .unwrap()
            .into()
    }

    fn test_timestamp_adding(seconds: u32) -> DateTime<FixedOffset> {
        Self::test_timestamp() + Duration::seconds(seconds as i64)
    }
}

pub struct ConnectedClient {
    pub client: Client<Arc<SQLiteCache>, NoopAvatarCache>,
    pub connection: Arc<test::Connection>,
    pub data_cache: Arc<SQLiteCache>,
    pub id_provider: Arc<IncrementingIDProvider>,
}

#[async_trait(?Send)]
impl ClientTestAdditions for Client<SQLiteCache, NoopAvatarCache> {
    async fn connected_client() -> Result<ConnectedClient> {
        Self::connected_client_with_time_provider(SystemTimeProvider::default()).await
    }

    async fn connected_client_with_time_provider<T: TimeProvider + 'static>(
        time_provider: T,
    ) -> Result<ConnectedClient> {
        let connection = Arc::new(test::Connection::default());
        let id_provider = Arc::new(IncrementingIDProvider::new());
        let data_cache = Arc::new(SQLiteCache::in_memory_cache());

        let client = ClientBuilder::new()
            .set_connector_provider(test::Connector::provider(connection.clone()))
            .set_data_cache(data_cache.clone())
            .set_avatar_cache(NoopAvatarCache::default())
            .set_id_provider(id_provider.clone() as Arc<dyn IDProvider>)
            .set_time_provider(time_provider)
            .set_software_version(SoftwareVersion {
                name: "prose-test-client".to_string(),
                version: "1.2.3".to_string(),
                os: Some("unknown os".to_string()),
            })
            .build();

        client
            .connect(
                &FullJid::from_str(&format!("{}/test", BareJid::ours()))?,
                "",
                Availability::Available,
            )
            .await?;

        id_provider.reset();
        connection.reset();

        Ok(ConnectedClient {
            client,
            connection,
            data_cache,
            id_provider,
        })
    }
}
