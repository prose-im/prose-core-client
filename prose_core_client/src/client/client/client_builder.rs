use crate::cache::AvatarCache;
use crate::client::client::client::ConnectorProvider;
use crate::{Client, ClientDelegate, DataCache, NoopAvatarCache, SQLiteCache};
use prose_core_lib::{
    IDProvider, LibstropheConnector, SystemTimeProvider, TimeProvider, UUIDProvider,
};
use std::sync::Arc;

pub struct ClientBuilder<D: DataCache + 'static, A: AvatarCache + 'static> {
    connector_provider: ConnectorProvider,
    data_cache: D,
    avatar_cache: A,
    delegate: Option<Box<dyn ClientDelegate>>,
    id_provider: Arc<dyn IDProvider>,
    time_provider: Arc<dyn TimeProvider>,
}

impl<D: DataCache, A: AvatarCache> ClientBuilder<D, A> {
    pub fn new() -> ClientBuilder<SQLiteCache, NoopAvatarCache> {
        ClientBuilder {
            connector_provider: Box::new(|| Box::new(LibstropheConnector::default())),
            data_cache: SQLiteCache::in_memory_cache(),
            avatar_cache: NoopAvatarCache::new(),
            delegate: None,
            id_provider: Arc::new(UUIDProvider::new()),
            time_provider: Arc::new(SystemTimeProvider::new()),
        }
    }

    pub fn set_connector_provider(mut self, connector_provider: ConnectorProvider) -> Self {
        self.connector_provider = connector_provider;
        self
    }

    pub fn set_data_cache<D2: DataCache>(self, data_cache: D2) -> ClientBuilder<D2, A> {
        ClientBuilder {
            connector_provider: self.connector_provider,
            data_cache,
            avatar_cache: self.avatar_cache,
            delegate: self.delegate,
            id_provider: self.id_provider,
            time_provider: self.time_provider,
        }
    }

    pub fn set_avatar_cache<A2: AvatarCache>(self, avatar_cache: A2) -> ClientBuilder<D, A2> {
        ClientBuilder {
            connector_provider: self.connector_provider,
            data_cache: self.data_cache,
            avatar_cache,
            delegate: self.delegate,
            id_provider: self.id_provider,
            time_provider: self.time_provider,
        }
    }

    pub fn set_delegate(mut self, delegate: Option<Box<dyn ClientDelegate>>) -> Self {
        self.delegate = delegate;
        self
    }

    pub fn set_id_provider<P: IDProvider + 'static>(mut self, id_provider: P) -> Self {
        self.id_provider = Arc::new(id_provider);
        self
    }

    pub fn set_time_provider<T: TimeProvider + 'static>(mut self, time_provider: T) -> Self {
        self.time_provider = Arc::new(time_provider);
        self
    }

    pub fn build(self) -> Client<D, A> {
        Client::new(
            self.connector_provider,
            self.data_cache,
            self.avatar_cache,
            self.delegate,
            self.id_provider,
            self.time_provider,
        )
    }
}
