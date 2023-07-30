use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, FixedOffset, TimeZone, Utc};
use jid::{BareJid, FullJid};
use std::str::FromStr;
use std::sync::Arc;

pub use incrementing_id_provider::IncrementingIDProvider;
pub use message_builder::MessageBuilder;
use prose_xmpp::{test, IDProvider};

use crate::types::Availability;
use crate::{
    avatar_cache::NoopAvatarCache, data_cache::sqlite::SQLiteCache, Client, ClientBuilder,
};

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

#[async_trait(?Send)]
pub trait ClientTestAdditions {
    async fn connected_client() -> Result<ConnectedClient>;
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
        let connection = Arc::new(test::Connection::default());
        let id_provider = Arc::new(IncrementingIDProvider::new());
        let data_cache = Arc::new(SQLiteCache::in_memory_cache());

        let client = ClientBuilder::new()
            .set_connector_provider(test::Connector::provider(connection.clone()))
            .set_data_cache(data_cache.clone())
            .set_avatar_cache(NoopAvatarCache::default())
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
