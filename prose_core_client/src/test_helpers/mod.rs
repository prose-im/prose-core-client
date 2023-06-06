use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};
use jid::{BareJid, FullJid};

pub use incrementing_id_provider::IncrementingIDProvider;
pub use message_builder::MessageBuilder;
use prose_core_domain::Availability;
use prose_core_lib::test_helpers::TestConnection;
use prose_core_lib::IDProvider;

use crate::{Client, ClientBuilder, NoopAvatarCache, SQLiteCache};

mod incrementing_id_provider;
mod message_builder;

pub trait BareJidTestAdditions {
    fn ours() -> BareJid;
    fn theirs() -> BareJid;
}

pub trait DateTimeTestAdditions {
    fn test_timestamp() -> DateTime<Utc>;
    fn test_timestamp_adding(seconds: u32) -> DateTime<Utc>;
}

#[async_trait]
pub trait ClientTestAdditions {
    async fn connected_client() -> anyhow::Result<ConnectedClient>;
}

impl BareJidTestAdditions for BareJid {
    fn ours() -> BareJid {
        BareJid {
            node: Some("test".to_string()),
            domain: "prose.org".to_string(),
        }
    }

    fn theirs() -> BareJid {
        BareJid {
            node: Some("friend".to_string()),
            domain: "prose.org".to_string(),
        }
    }
}

impl DateTimeTestAdditions for Utc {
    fn test_timestamp() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2023, 06, 02, 17, 00, 00).unwrap()
    }

    fn test_timestamp_adding(seconds: u32) -> DateTime<Utc> {
        Self::test_timestamp() + Duration::seconds(seconds as i64)
    }
}

pub struct ConnectedClient {
    pub client: Client<Arc<SQLiteCache>, NoopAvatarCache>,
    pub connection: Arc<TestConnection>,
    pub data_cache: Arc<SQLiteCache>,
    pub id_provider: Arc<IncrementingIDProvider>,
}

#[async_trait]
impl ClientTestAdditions for Client<SQLiteCache, NoopAvatarCache> {
    async fn connected_client() -> anyhow::Result<ConnectedClient> {
        let connection = TestConnection::new();
        let connection_clone = connection.clone();
        let id_provider = Arc::new(IncrementingIDProvider::new());
        let data_cache = Arc::new(SQLiteCache::in_memory_cache());

        let client = ClientBuilder::<SQLiteCache, NoopAvatarCache>::new()
            .set_connector_provider(Box::new(move || connection_clone.connector()))
            .set_data_cache(data_cache.clone())
            .set_id_provider(id_provider.clone() as Arc<dyn IDProvider>)
            .build();

        client
            .connect(
                &FullJid::from_str(&format!("{}/test", BareJid::ours()))?,
                "",
                Availability::Available,
                None,
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
