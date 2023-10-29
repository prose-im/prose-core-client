// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use constant_time_provider::ConstantTimeProvider;
pub use message_builder::MessageBuilder;
pub use mock_app_dependencies::MockAppDependencies;
use prose_xmpp::test::BareJidTestAdditions;

// use crate::data_cache::indexed_db::PlatformCache;
// use crate::types::{Availability, SoftwareVersion};
// use crate::{avatar_cache::NoopAvatarCache, Client, ClientBuilder};

mod constant_time_provider;
mod message_builder;
mod mock_app_dependencies;

pub mod mock_data {
    pub use super::mock_app_dependencies::{
        mock_account_jid as account_jid, mock_muc_service as muc_service,
        mock_reference_date as reference_date,
    };
}

// #[async_trait(?Send)]
// pub trait ClientTestAdditions {
//     async fn connected_client() -> Result<ConnectedClient>;
//     async fn connected_client_with_time_provider<T: TimeProvider + 'static>(
//         time_provider: T,
//     ) -> Result<ConnectedClient>;
// }

// pub struct ConnectedClient {
//     pub client: Client<Arc<PlatformCache>, NoopAvatarCache>,
//     pub connection: Arc<test::Connection>,
//     pub data_cache: Arc<PlatformCache>,
//     pub id_provider: Arc<IncrementingIDProvider>,
// }
//
// #[async_trait(?Send)]
// impl ClientTestAdditions for Client<PlatformCache, NoopAvatarCache> {
//     async fn connected_client() -> Result<ConnectedClient> {
//         Self::connected_client_with_time_provider(SystemTimeProvider::default()).await
//     }
//
//     async fn connected_client_with_time_provider<T: TimeProvider + 'static>(
//         time_provider: T,
//     ) -> Result<ConnectedClient> {
//         let connection = Arc::new(test::Connection::default());
//         let id_provider = Arc::new(IncrementingIDProvider::new());
//         let data_cache = Arc::new(PlatformCache::temporary_cache().await?);
//
//         connection.use_start_sequence_handler();
//
//         let client = ClientBuilder::new()
//             .set_connector_provider(test::Connector::provider(connection.clone()))
//             .set_data_cache(data_cache.clone())
//             .set_avatar_cache(NoopAvatarCache::default())
//             .set_id_provider(id_provider.clone() as Arc<dyn IDProvider>)
//             .set_time_provider(time_provider)
//             .set_software_version(SoftwareVersion {
//                 name: "prose-test-client".to_string(),
//                 version: "1.2.3".to_string(),
//                 os: Some("unknown os".to_string()),
//             })
//             .build();
//
//         client
//             .connect(
//                 &FullJid::from_str(&format!("{}/test", BareJid::ours()))?,
//                 "",
//                 Availability::Available,
//             )
//             .await?;
//
//         id_provider.reset();
//         connection.reset();
//
//         Ok(ConnectedClient {
//             client,
//             connection,
//             data_cache,
//             id_provider,
//         })
//     }
// }
