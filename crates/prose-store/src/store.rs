use crate::{
    driver::Driver, Database, KeyTuple, KeyType, ReadTransaction, ReadableCollection,
    VersionChangeEvent, WritableCollection, WriteTransaction,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::Deref;
use std::sync::Arc;

pub struct Store<D: Driver> {
    db: Arc<D::Database>,
}

impl<D: Driver> Clone for Store<D> {
    fn clone(&self) -> Self {
        Store {
            db: self.db.clone(),
        }
    }
}

impl<D: Driver> Store<D> {
    pub async fn open<F>(driver: D, version: u32, update_handler: F) -> Result<Self, D::Error>
    where
        F: Fn(&VersionChangeEvent<D::UpgradeTransaction<'_>>) -> Result<(), D::Error>
            + Send
            + 'static,
    {
        assert!(version > 0, "`version` must be greater 0");

        Ok(Self {
            db: Arc::new(driver.open(version, update_handler).await?),
        })
    }
}

impl<D: Driver> Deref for Store<D> {
    type Target = D::Database;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl<D: Driver> Store<D> {
    pub async fn truncate_collections(&self, collection_names: &[&str]) -> Result<(), D::Error> {
        let tx = self
            .db
            .transaction_for_reading_and_writing(collection_names)
            .await?;
        tx.truncate_collections(collection_names)
    }

    pub async fn truncate_all_collections(&self) -> Result<(), D::Error> {
        self.truncate_collections(
            self.db
                .collection_names()
                .await?
                .iter()
                .map(|s| s.as_ref())
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .await
    }
}

impl<D: Driver> Store<D> {
    pub async fn set<K: KeyType + ?Sized, V: Serialize + ?Sized + Send + Sync>(
        &self,
        collection_name: &str,
        key: &K,
        value: &V,
    ) -> Result<(), D::Error> {
        let tx = self
            .db
            .transaction_for_reading_and_writing(&[collection_name])
            .await?;
        let collection = tx.writeable_collection(collection_name)?;
        collection.set(key, value).await
    }

    pub async fn put<K: KeyType + ?Sized, V: Serialize>(
        &self,
        collection_name: &str,
        key: &K,
        value: &V,
    ) -> Result<(), D::Error> {
        let tx = self
            .db
            .transaction_for_reading_and_writing(&[collection_name])
            .await?;
        let collection = tx.writeable_collection(collection_name)?;
        collection.put(key, value)
    }

    pub async fn delete<K: KeyType + ?Sized>(
        &self,
        collection_name: &str,
        key: &K,
    ) -> Result<(), D::Error> {
        let tx = self
            .db
            .transaction_for_reading_and_writing(&[collection_name])
            .await?;
        let collection = tx.writeable_collection(collection_name)?;
        collection.delete(key)
    }

    pub async fn get<K: KeyTuple + ?Sized, V: DeserializeOwned>(
        &self,
        collection_name: &str,
        key: &K,
    ) -> Result<Option<V>, D::Error> {
        let tx = self.db.transaction_for_reading(&[collection_name]).await?;
        let collection = tx.readable_collection(collection_name)?;
        collection.get(key).await
    }

    pub async fn contains_key<K: KeyTuple + ?Sized>(
        &self,
        collection_name: &str,
        key: &K,
    ) -> Result<bool, D::Error> {
        let tx = self.db.transaction_for_reading(&[collection_name]).await?;
        let collection = tx.readable_collection(collection_name)?;
        collection.contains_key(key).await
    }
}

#[macro_export]
macro_rules! upsert {
    ($entity:ident, store: $store:expr, id: $id:expr, insert_if_needed: $insert_closure:expr, update: $update_closure:expr) => {{
        let tx = $store
            .transaction_for_reading_and_writing(&[$entity::collection()])
            .await?;
        {
            let collection = tx.writeable_collection($entity::collection())?;
            let mut value = collection
                .get::<_, _>($id)
                .await?
                .unwrap_or_else($insert_closure);
            $update_closure(&mut value);
            collection.put_entity(&value)?;
        }
        tx.commit().await?;
    }};
}
