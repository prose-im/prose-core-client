use crate::driver::{Driver, ReadMode, ReadOnly, ReadWrite, WriteMode};
use crate::{
    Collection, Database, IndexSpec, IndexedCollection, KeyType, Query, QueryDirection,
    ReadTransaction, ReadableCollection, StoreError, Transaction, UpgradeTransaction,
    VersionChangeEvent, WritableCollection, WriteTransaction,
};
use async_trait::async_trait;
use gloo_utils::format::JsValueSerdeExt;
use indexed_db_futures::prelude::*;
use indexed_db_futures::web_sys::{DomException, IdbKeyRange};
use prose_wasm_utils::SendUnlessWasm;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Bound;
use wasm_bindgen::JsValue;

pub struct IndexedDBDriver {
    db_name: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Database is closed")]
    Closed,

    #[error("DomException {name}: {message}")]
    DomException { name: String, message: String },

    #[error(transparent)]
    JSON(#[from] serde_json::Error),

    #[error("Invalid DB Key")]
    InvalidDBKey,

    #[error("Duplicate Key")]
    DuplicateKey,

    #[error("[IndexedDB] {0}")]
    IndexedDB(String),
}

impl StoreError for Error {}

impl From<DomException> for Error {
    fn from(value: DomException) -> Self {
        Self::DomException {
            name: value.name(),
            message: value.message(),
        }
    }
}

impl IndexedDBDriver {
    pub fn new(db_name: impl AsRef<str>) -> Self {
        IndexedDBDriver {
            db_name: db_name.as_ref().to_string(),
        }
    }
}

#[async_trait(? Send)]
impl Driver for IndexedDBDriver {
    type Error = Error;

    type UpgradeTransaction<'db> = IndexedDBUpgradeTransaction<'db>;
    type Database = IndexedDB;

    async fn open<F>(self, version: u32, update_handler: F) -> Result<Self::Database, Self::Error>
    where
        F: Fn(&VersionChangeEvent<Self::UpgradeTransaction<'_>>) -> Result<(), Self::Error>
            + Send
            + 'static,
    {
        let mut db_req = IdbDatabase::open_u32(&self.db_name, version)?;
        db_req.set_on_upgrade_needed(Some(move |change_event: &IdbVersionChangeEvent| {
            let db = change_event.db();

            let change_event = VersionChangeEvent {
                tx: IndexedDBUpgradeTransaction { db },
                old_version: change_event.old_version() as u32,
                new_version: change_event.new_version() as u32,
                phantom: Default::default(),
            };
            update_handler(&change_event).map_err(|err| JsValue::from_str(&err.to_string()))?;
            Ok(())
        }));

        Ok(IndexedDB {
            db: db_req.into_future().await?,
        })
    }
}

pub struct IndexedDB {
    db: IdbDatabase,
}

#[async_trait(? Send)]
impl Database for IndexedDB {
    type Error = Error;

    type ReadTransaction<'db> = IndexedDBTransaction<'db, ReadOnly> where Self: 'db;
    type ReadWriteTransaction<'db> = IndexedDBTransaction<'db, ReadWrite> where Self: 'db;

    async fn collection_names(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.db.object_store_names().collect())
    }

    async fn transaction_for_reading(
        &self,
        stores: &[&str],
    ) -> Result<Self::ReadTransaction<'_>, Self::Error> {
        IndexedDBTransaction::new(&self.db, stores, IdbTransactionMode::Readonly)
    }

    async fn transaction_for_reading_and_writing(
        &self,
        stores: &[&str],
    ) -> Result<Self::ReadWriteTransaction<'_>, Self::Error> {
        IndexedDBTransaction::new(&self.db, stores, IdbTransactionMode::Readwrite)
    }
}

pub struct IndexedDBUpgradeTransaction<'db> {
    db: &'db IdbDatabase,
}

impl<'db> UpgradeTransaction<'db> for IndexedDBUpgradeTransaction<'db> {
    type Error = Error;
    type ReadWriteTransaction<'tx> = IndexedDBTransaction<'tx, ReadWrite> where Self: 'tx;

    fn collection_names(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.db.object_store_names().collect())
    }

    fn create_collection(
        &self,
        name: &str,
    ) -> Result<
        <Self::ReadWriteTransaction<'_> as WriteTransaction<'_>>::WritableCollection<'_>,
        Self::Error,
    > {
        let store = self.db.create_object_store(name)?;
        Ok(IndexedDBCollection::new(store))
    }

    fn delete_collection(&self, name: &str) -> Result<(), Self::Error> {
        self.db.delete_object_store(name)?;
        Ok(())
    }
}

pub struct IndexedDBTransaction<'db, Mode> {
    tx: IdbTransaction<'db>,
    phantom: PhantomData<Mode>,
}

impl<'db, Mode> IndexedDBTransaction<'db, Mode> {
    fn new(db: &'db IdbDatabase, stores: &[&str], mode: IdbTransactionMode) -> Result<Self, Error> {
        Ok(Self {
            tx: db.transaction_on_multi_with_mode(stores, mode)?,
            phantom: Default::default(),
        })
    }
}

impl<'db, Mode> Transaction<'db> for IndexedDBTransaction<'db, Mode> {
    type Error = Error;
}

impl<'db, Mode> ReadTransaction<'db> for IndexedDBTransaction<'db, Mode>
where
    Mode: ReadMode,
{
    type ReadableCollection<'tx> = IndexedDBCollection<'tx, IdbObjectStore<'tx>, ReadOnly> where Self: 'tx;

    fn readable_collection(&self, name: &str) -> Result<Self::ReadableCollection<'_>, Self::Error> {
        Ok(IndexedDBCollection::new(self.tx.object_store(name)?))
    }
}

#[async_trait(? Send)]
impl<'db, Mode> WriteTransaction<'db> for IndexedDBTransaction<'db, Mode>
where
    Mode: WriteMode,
{
    type WritableCollection<'tx> = IndexedDBCollection<'tx, IdbObjectStore<'tx>, ReadWrite> where Self: 'tx;

    fn writeable_collection(
        &self,
        name: &str,
    ) -> Result<Self::WritableCollection<'_>, Self::Error> {
        Ok(IndexedDBCollection::new(self.tx.object_store(name)?))
    }

    async fn commit(self) -> Result<(), Self::Error> {
        self.tx.await.into_result()?;
        Ok(())
    }

    async fn rollback(self) -> Result<(), Self::Error> {
        self.tx.abort()?;
        Ok(())
    }
}

pub struct IndexedDBCollection<'tx, QuerySource: IdbQuerySource, Mode> {
    store: QuerySource,
    phantom: PhantomData<&'tx Mode>,
}

impl<'tx, QuerySource: IdbQuerySource, Mode> IndexedDBCollection<'tx, QuerySource, Mode> {
    fn new(store: QuerySource) -> Self {
        Self {
            store,
            phantom: Default::default(),
        }
    }
}

impl<'tx, QuerySource: IdbQuerySource, Mode> Collection<'tx>
    for IndexedDBCollection<'tx, QuerySource, Mode>
{
    type Error = Error;
}

#[async_trait(? Send)]
impl<'tx, Mode> IndexedCollection<'tx> for IndexedDBCollection<'tx, IdbObjectStore<'tx>, Mode>
where
    Mode: ReadMode,
{
    type Index<'coll> = IndexedDBCollection<'coll, IdbIndex<'coll>, ReadOnly> where Self: 'coll;

    fn index(&self, name: &str) -> Result<Self::Index<'_>, Self::Error> {
        Ok(IndexedDBCollection::new(self.store.index(name)?))
    }
}

#[async_trait(? Send)]
impl<'tx, QuerySource: IdbQuerySource, Mode> ReadableCollection<'tx>
    for IndexedDBCollection<'tx, QuerySource, Mode>
where
    Mode: ReadMode,
{
    async fn get<K: KeyType + ?Sized, V: DeserializeOwned>(
        &self,
        key: &K,
    ) -> Result<Option<V>, Self::Error> {
        let value: Option<V> = self
            .store
            .get(&key.to_js_value()?)?
            .await?
            .map(|value| JsValueSerdeExt::into_serde(&value))
            .transpose()?;
        Ok(value)
    }

    async fn contains_key<K: KeyType + ?Sized>(&self, key: &K) -> Result<bool, Self::Error> {
        let contains_key = self.store.get_key(&key.to_js_value()?)?.await?.is_some();
        Ok(contains_key)
    }

    async fn all_keys(&self) -> Result<Vec<String>, Self::Error> {
        let keys = self.store.get_all_keys()?.await?;
        keys.into_iter()
            .map(|key| key.as_string().ok_or(Error::InvalidDBKey))
            .collect()
    }

    async fn get_all<Value: DeserializeOwned + Send>(
        &self,
        query: Query<impl KeyType>,
        direction: QueryDirection,
        limit: Option<usize>,
    ) -> Result<Vec<(String, Value)>, Self::Error> {
        self.get_all_filtered(query, direction, limit, |key, value| Some((key, value)))
            .await
    }

    async fn get_all_filtered<Value: DeserializeOwned + Send, T: Send>(
        &self,
        query: Query<impl KeyType>,
        direction: QueryDirection,
        limit: Option<usize>,
        mut filter: impl FnMut(String, Value) -> Option<T> + SendUnlessWasm,
    ) -> Result<Vec<T>, Self::Error> {
        let range: Option<IdbKeyRange> = query.try_into()?;
        let direction = match direction {
            QueryDirection::Forward => IdbCursorDirection::Next,
            QueryDirection::Backward => IdbCursorDirection::Prev,
        };

        let cursor = if let Some(range) = range {
            self.store
                .open_cursor_with_range_and_direction(&range, direction)
        } else {
            self.store.open_cursor_with_direction(direction)
        }?
        .await?;

        let Some(cursor) = cursor else {
            return Ok(vec![]);
        };

        let mut result = vec![];
        let limit = limit.unwrap_or(usize::MAX);

        if limit == 0 {
            return Ok(vec![]);
        }

        while result.len() <= limit {
            let key = cursor
                .primary_key()
                .and_then(|key| key.as_string())
                .ok_or(Error::InvalidDBKey)?;
            let value = JsValueSerdeExt::into_serde(&cursor.value())?;

            if let Some(transformed_value) = (filter)(key, value) {
                result.push(transformed_value);
            }

            if result.len() == limit || !cursor.continue_cursor()?.await? {
                break;
            }
        }

        Ok(result)
    }
}

#[async_trait(? Send)]
impl<'tx> WritableCollection<'tx> for IndexedDBCollection<'tx, IdbObjectStore<'tx>, ReadWrite> {
    fn add_index(&self, idx: IndexSpec) -> Result<(), Self::Error> {
        self.store.create_index_with_params(
            &idx.key,
            &IdbKeyPath::str(&idx.key),
            IdbIndexParameters::new().unique(idx.unique),
        )?;

        Ok(())
    }

    async fn set<K: KeyType + ?Sized, V: Serialize + ?Sized>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        let key = key.to_js_value()?;

        // We mimic SQLite behavior here which raises an error immediately when a duplicate key is
        // inserted, unlike IndexedDB which does not raise an error until the transaction
        // is committed.
        if self.store.get_key(&key)?.await?.is_some() {
            return Err(Error::DuplicateKey);
        }

        self.store
            .add_key_val(&key, &<JsValue as JsValueSerdeExt>::from_serde(value)?)?;
        Ok(())
    }

    fn put<K: KeyType + ?Sized, V: Serialize>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        self.store.put_key_val(
            &key.to_js_value()?,
            &<JsValue as JsValueSerdeExt>::from_serde(value)?,
        )?;
        Ok(())
    }

    fn delete<K: KeyType + ?Sized>(&self, key: &K) -> Result<(), Self::Error> {
        self.store.delete(&key.to_js_value()?)?;
        Ok(())
    }

    fn truncate(&self) -> Result<(), Self::Error> {
        self.store.clear()?;
        Ok(())
    }
}

impl<T: KeyType> TryFrom<Query<T>> for Option<IdbKeyRange> {
    type Error = Error;

    fn try_from(value: Query<T>) -> Result<Option<IdbKeyRange>, Self::Error> {
        let result = match value {
            Query::All => None,

            Query::Range {
                start: Bound::Included(start),
                end: Bound::Included(end),
            } => Some(
                IdbKeyRange::bound_with_lower_open_and_upper_open(
                    &start.to_js_value()?,
                    &end.to_js_value()?,
                    false,
                    false,
                )
                .map_err(|_| {
                    Error::IndexedDB(format!(
                        "Failed to build IdbKeyRange::bound (false, false) from {:?}/{:?}",
                        start, end
                    ))
                })?,
            ),
            Query::Range {
                start: Bound::Included(start),
                end: Bound::Excluded(end),
            } => Some(
                IdbKeyRange::bound_with_lower_open_and_upper_open(
                    &start.to_js_value()?,
                    &end.to_js_value()?,
                    false,
                    true,
                )
                .map_err(|_| {
                    Error::IndexedDB(format!(
                        "Failed to build IdbKeyRange::bound (false, true) from {:?}/{:?}",
                        start, end
                    ))
                })?,
            ),
            Query::Range {
                start: Bound::Included(start),
                end: Bound::Unbounded,
            } => Some(
                IdbKeyRange::lower_bound_with_open(&start.to_js_value()?, false).map_err(|_| {
                    Error::IndexedDB(format!(
                        "Failed to build IdbKeyRange::lowerBound (false) from {:?}",
                        start
                    ))
                })?,
            ),

            Query::Range {
                start: Bound::Excluded(start),
                end: Bound::Included(end),
            } => Some(
                IdbKeyRange::bound_with_lower_open_and_upper_open(
                    &start.to_js_value()?,
                    &end.to_js_value()?,
                    true,
                    false,
                )
                .map_err(|_| {
                    Error::IndexedDB(format!(
                        "Failed to build IdbKeyRange::bound (true, false) from {:?}/{:?}",
                        start, end
                    ))
                })?,
            ),
            Query::Range {
                start: Bound::Excluded(start),
                end: Bound::Excluded(end),
            } => Some(
                IdbKeyRange::bound_with_lower_open_and_upper_open(
                    &start.to_js_value()?,
                    &end.to_js_value()?,
                    true,
                    true,
                )
                .map_err(|_| {
                    Error::IndexedDB(format!(
                        "Failed to build IdbKeyRange::bound (true, true) from {:?}/{:?}",
                        start, end
                    ))
                })?,
            ),
            Query::Range {
                start: Bound::Excluded(start),
                end: Bound::Unbounded,
            } => Some(
                IdbKeyRange::lower_bound_with_open(&start.to_js_value()?, true).map_err(|_| {
                    Error::IndexedDB(format!(
                        "Failed to build IdbKeyRange::lowerBound (true) from {:?}",
                        start
                    ))
                })?,
            ),

            Query::Range {
                start: Bound::Unbounded,
                end: Bound::Included(end),
            } => Some(
                IdbKeyRange::upper_bound_with_open(&end.to_js_value()?, false).map_err(|_| {
                    Error::IndexedDB(format!(
                        "Failed to build IdbKeyRange::upperBound (false) from {:?}",
                        false
                    ))
                })?,
            ),
            Query::Range {
                start: Bound::Unbounded,
                end: Bound::Excluded(end),
            } => Some(
                IdbKeyRange::upper_bound_with_open(&end.to_js_value()?, true).map_err(|_| {
                    Error::IndexedDB(format!(
                        "Failed to build IdbKeyRange::upperBound (false) from {:?}",
                        false
                    ))
                })?,
            ),
            Query::Range {
                start: Bound::Unbounded,
                end: Bound::Unbounded,
            } => None,

            Query::Only(value) => Some(IdbKeyRange::only(&value.to_js_value()?).map_err(|_| {
                Error::IndexedDB(format!(
                    "Failed to build IdbKeyRange::only from {:?}",
                    value
                ))
            })?),
        };

        Ok(result)
    }
}

trait KeyTypeExt {
    fn to_js_value(&self) -> Result<JsValue, Error>;
}

impl<T: KeyType + ?Sized> KeyTypeExt for T {
    fn to_js_value(&self) -> Result<JsValue, Error> {
        <JsValue as JsValueSerdeExt>::from_serde(&self.to_raw_key())
            .map_err(|_| Error::IndexedDB(format!("Failed to convert {:?} to a JsValue.", self)))
    }
}
