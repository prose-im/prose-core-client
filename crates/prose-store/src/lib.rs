use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Local, NaiveDate, Utc};
use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};

mod driver;
pub mod prelude;
mod store;

pub trait StoreError: Error {}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait Database: SendUnlessWasm + SyncUnlessWasm {
    type Error: StoreError;

    type ReadTransaction<'db>: ReadTransaction<'db, Error = Self::Error>
    where
        Self: 'db;
    type ReadWriteTransaction<'db>: WriteTransaction<'db, Error = Self::Error>
        + ReadTransaction<'db, Error = Self::Error>
    where
        Self: 'db;

    /// Returns the name of all collections in the database
    async fn collection_names(&self) -> Result<Vec<String>, Self::Error>;

    async fn transaction_for_reading(
        &self,
        collections: &[&str],
    ) -> Result<Self::ReadTransaction<'_>, Self::Error>;

    async fn transaction_for_reading_and_writing(
        &self,
        collections: &[&str],
    ) -> Result<Self::ReadWriteTransaction<'_>, Self::Error>;
}

pub struct VersionChangeEvent<'db, Tx: UpgradeTransaction<'db>> {
    pub tx: Tx,
    pub old_version: u32,
    pub new_version: u32,
    phantom: PhantomData<&'db Tx>,
}

pub trait UpgradeTransaction<'db> {
    type Error: StoreError;

    type ReadWriteTransaction<'tx>: WriteTransaction<'tx, Error = Self::Error>
        + ReadTransaction<'tx, Error = Self::Error>
    where
        Self: 'tx;

    /// Returns the name of all collections in the database
    fn collection_names(&self) -> Result<Vec<String>, Self::Error>;

    fn create_collection(
        &self,
        name: &str,
    ) -> Result<
        <Self::ReadWriteTransaction<'_> as WriteTransaction<'_>>::WritableCollection<'_>,
        Self::Error,
    >;

    fn delete_collection(&self, name: &str) -> Result<(), Self::Error>;
}

pub trait Transaction<'tx>: SendUnlessWasm + SyncUnlessWasm {
    type Error: StoreError;
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait WriteTransaction<'db>: Transaction<'db> {
    type WritableCollection<'a>: WritableCollection<'a, Error = Self::Error>
        + ReadableCollection<'a, Error = Self::Error>
        + IndexedCollection<'a, Error = Self::Error>
    where
        Self: 'a;

    fn writeable_collection(&self, name: &str)
        -> Result<Self::WritableCollection<'_>, Self::Error>;

    async fn commit(self) -> Result<(), Self::Error>;
    async fn rollback(self) -> Result<(), Self::Error>;

    fn truncate_collections(&self, collection_names: &[&str]) -> Result<(), Self::Error> {
        for collection_name in collection_names {
            let collection = self.writeable_collection(collection_name)?;
            collection.truncate()?;
        }
        Ok(())
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait ReadTransaction<'db>: Transaction<'db> {
    type ReadableCollection<'tx>: ReadableCollection<'tx, Error = Self::Error>
        + IndexedCollection<'tx, Error = Self::Error>
    where
        Self: 'tx;

    fn readable_collection(&self, name: &str) -> Result<Self::ReadableCollection<'_>, Self::Error>;
}

pub trait Collection<'tx>: SendUnlessWasm + SyncUnlessWasm {
    type Error: StoreError;
}

pub trait IndexedCollection<'tx>: Collection<'tx> {
    type Index<'coll>: ReadableCollection<'coll, Error = Self::Error>
    where
        Self: 'coll;

    fn index(&self, name: &str) -> Result<Self::Index<'_>, Self::Error>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait ReadableCollection<'tx>: Collection<'tx> {
    async fn get<K: KeyType + ?Sized, V: DeserializeOwned>(
        &self,
        key: &K,
    ) -> Result<Option<V>, Self::Error>;

    async fn contains_key<K: KeyType + ?Sized>(&self, key: &K) -> Result<bool, Self::Error>;
    async fn all_keys(&self) -> Result<Vec<String>, Self::Error>;

    /// Collects all items matching `query`.
    ///
    /// NB: Implementing a Cursor is currently not possible because we cannot pull a
    /// `rusqlite::Statement` and `rusqlite::Rows` out of the `deadpool::SyncWrapper` but the
    /// IdbCursor can only be iterated asynchronously.
    async fn get_all<Value: DeserializeOwned + Send>(
        &self,
        query: Query<impl KeyType>,
        direction: QueryDirection,
        limit: Option<usize>,
    ) -> Result<Vec<(String, Value)>, Self::Error>;

    async fn get_all_filtered<Value: DeserializeOwned + Send, T: Send>(
        &self,
        query: Query<impl KeyType>,
        direction: QueryDirection,
        limit: Option<usize>,
        filter: impl FnMut(String, Value) -> Option<T> + SendUnlessWasm,
    ) -> Result<Vec<T>, Self::Error>;

    async fn get_all_values<Value: DeserializeOwned + Send>(
        &self,
        query: Query<impl KeyType>,
        direction: QueryDirection,
        limit: Option<usize>,
    ) -> Result<Vec<Value>, Self::Error> {
        self.get_all_filtered(query, direction, limit, |_, value| Some(value))
            .await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait WritableCollection<'tx>: Collection<'tx> {
    fn add_index(&self, idx: IndexSpec) -> Result<(), Self::Error>;

    async fn set<K: KeyType + ?Sized, V: Serialize + ?Sized + Send + Sync>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>;
    fn put<K: KeyType + ?Sized, V: Serialize>(&self, key: &K, value: &V)
        -> Result<(), Self::Error>;
    fn delete<K: KeyType + ?Sized>(&self, key: &K) -> Result<(), Self::Error>;

    /// Deletes all entries in the collection.
    fn truncate(&self) -> Result<(), Self::Error>;
}

pub trait ReadWriteTransaction<'tx>: ReadTransaction<'tx> + WriteTransaction<'tx> {}

pub struct IndexSpec {
    pub key: String,
    pub unique: bool,
}

pub struct IndexSpecBuilder {
    key: String,
    unique: bool,
}

impl IndexSpec {
    /// Creates a new index with the name `name`. Note that the name must match the name of a field
    /// of the `Collection`'s type.
    pub fn builder(key: impl Into<String>) -> IndexSpecBuilder {
        IndexSpecBuilder {
            key: key.into(),
            unique: false,
        }
    }
}

impl IndexSpecBuilder {
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }
}

impl IndexSpecBuilder {
    pub fn build(self) -> IndexSpec {
        IndexSpec {
            key: self.key,
            unique: self.unique,
        }
    }
}

#[derive(Default)]
pub enum QueryDirection {
    #[default]
    Forward,
    Backward,
}

#[cfg(target_arch = "wasm32")]
pub trait KeyType: std::fmt::Display + Serialize + std::fmt::Debug {}
#[cfg(not(target_arch = "wasm32"))]
pub trait KeyType: std::fmt::Display + Serialize + rusqlite::ToSql + Send + Sync {}

impl KeyType for i32 {}
impl KeyType for i64 {}
impl KeyType for f32 {}
impl KeyType for f64 {}
impl KeyType for String {}
impl KeyType for &String {}
impl KeyType for str {}
impl KeyType for &str {}
impl KeyType for NaiveDate {}
impl KeyType for DateTime<Local> {}
impl KeyType for DateTime<Utc> {}
impl KeyType for DateTime<FixedOffset> {}

pub enum Query<T: KeyType> {
    Range { start: Bound<T>, end: Bound<T> },
    Only(T),
}

impl<T: KeyType + Clone> Query<T> {
    pub fn from_range<B: RangeBounds<T>>(range: B) -> Self {
        Self::Range {
            start: range.start_bound().cloned(),
            end: range.end_bound().cloned(),
        }
    }
}
