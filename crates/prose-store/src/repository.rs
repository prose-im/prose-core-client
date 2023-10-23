use crate::prelude::{Driver, Store};
use crate::{
    Database, IndexSpec, KeyType, Query, QueryDirection, ReadTransaction, ReadableCollection,
    WritableCollection, WriteTransaction,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;

pub trait Entity: Serialize + DeserializeOwned + Send + Sync {
    type ID: KeyType;

    fn id(&self) -> &Self::ID;
    fn collection() -> &'static str;
    fn indexes() -> Vec<IndexSpec> {
        vec![]
    }
}

pub struct Repository<D: Driver, E: Entity> {
    store: Store<D>,
    phantom: PhantomData<E>,
}

impl<D: Driver, E: Entity> Repository<D, E> {
    pub fn new(store: Store<D>) -> Self {
        Self {
            store,
            phantom: Default::default(),
        }
    }

    pub async fn get(&self, id: &E::ID) -> Result<Option<E>, D::Error> {
        self.store.get(E::collection(), id).await
    }

    pub async fn get_all(&self) -> Result<Vec<E>, D::Error> {
        let tx = self
            .store
            .transaction_for_reading(&[E::collection()])
            .await?;
        let collection = tx.readable_collection(E::collection())?;
        collection
            .get_all_values(Query::<E::ID>::All, QueryDirection::Forward, None)
            .await
    }

    pub async fn put(&self, entity: &E) -> Result<(), D::Error> {
        self.store.put(E::collection(), entity.id(), &entity).await
    }

    pub async fn delete(&self, id: &E::ID) -> Result<(), D::Error> {
        self.store.delete(E::collection(), id).await
    }

    pub fn entry<'r, 'k>(&'r self, id: &'k E::ID) -> Entry<'r, 'k, D, E>
    where
        E::ID: Clone,
    {
        Entry {
            store: &self.store,
            key: &id,
            value: None,
        }
    }
}

impl<D: Driver, E: Entity> Repository<D, E> {
    pub fn store(&self) -> &Store<D> {
        &self.store
    }

    pub fn collection_name(&self) -> &str {
        E::collection()
    }
}

pub struct Entry<'r, 'k, D: Driver, E: Entity> {
    store: &'r Store<D>,
    key: &'k E::ID,
    value: Option<Box<dyn FnOnce(&'k E::ID) -> E + Send + Sync>>,
}

impl<'r, 'k, D: Driver, E: Entity + 'static> Entry<'r, 'k, D, E> {
    pub fn insert_if_needed(self, value: E) -> Self {
        self.insert_if_needed_with(|_| value)
    }

    pub fn insert_default_if_needed(self) -> Self
    where
        E: Default,
    {
        self.insert_if_needed_with(|_| E::default())
    }

    pub fn insert_if_needed_with<F>(self, f: F) -> Self
    where
        F: FnOnce(&'k E::ID) -> E + 'static + Send + Sync,
    {
        Self {
            store: self.store,
            key: self.key,
            value: Some(Box::new(f)),
        }
    }

    pub async fn and_update<F>(self, f: F) -> Result<(), D::Error>
    where
        F: FnOnce(&mut E) + 'static + Send + Sync,
    {
        let tx = self
            .store
            .transaction_for_reading_and_writing(&[E::collection()])
            .await?;

        {
            let collection = tx.writeable_collection(E::collection())?;
            let Some(mut value) = collection
                .get::<_, E>(self.key)
                .await?
                .or_else(|| self.value.map(|f| (f)(&self.key)))
            else {
                return Ok(());
            };
            (f)(&mut value);

            assert_eq!(
                value.id(),
                self.key,
                "Attempted to change the key of an entity in an update."
            );

            collection.put(self.key, &value)?;
        }
        tx.commit().await?;

        Ok(())
    }
}
