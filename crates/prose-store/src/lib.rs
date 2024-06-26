extern crate core;

use std::error::Error;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Serialize, Serializer};

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::repository::Entity;

mod driver;
mod entity_macro;
pub mod prelude;
mod repository;
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

    fn index(&self, columns: &[&str]) -> Result<Self::Index<'_>, Self::Error>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
pub trait ReadableCollection<'tx>: Collection<'tx> {
    async fn get<K: KeyTuple + ?Sized, V: DeserializeOwned>(
        &self,
        key: &K,
    ) -> Result<Option<V>, Self::Error>;

    async fn contains_key<K: KeyTuple + ?Sized>(&self, key: &K) -> Result<bool, Self::Error>;
    async fn all_keys(&self) -> Result<Vec<String>, Self::Error>;

    /// Collects all items matching `query`.
    ///
    /// NB: Implementing a Cursor is currently not possible because we cannot pull a
    /// `rusqlite::Statement` and `rusqlite::Rows` out of the `deadpool::SyncWrapper` but the
    /// IdbCursor can only be iterated asynchronously.
    async fn get_all<Value: DeserializeOwned + Send>(
        &self,
        query: Query<impl KeyTuple>,
        direction: QueryDirection,
        limit: Option<usize>,
    ) -> Result<Vec<(String, Value)>, Self::Error>;

    async fn get_all_filtered<Value: DeserializeOwned + Send, T: Send>(
        &self,
        query: Query<impl KeyTuple>,
        direction: QueryDirection,
        limit: Option<usize>,
        filter: impl FnMut(String, Value) -> Option<T> + SendUnlessWasm,
    ) -> Result<Vec<T>, Self::Error>;

    async fn get_all_values<Value: DeserializeOwned + Send>(
        &self,
        query: Query<impl KeyTuple>,
        direction: QueryDirection,
        limit: Option<usize>,
    ) -> Result<Vec<Value>, Self::Error> {
        self.get_all_filtered(query, direction, limit, |_, value| Some(value))
            .await
    }

    async fn fold<Value: DeserializeOwned + Send, T: Send>(
        &self,
        query: Query<impl KeyTuple>,
        init: T,
        f: impl FnMut(T, (String, Value)) -> T + SendUnlessWasm,
    ) -> Result<T, Self::Error>;
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
    async fn delete<K: KeyTuple + ?Sized>(&self, key: &K) -> Result<(), Self::Error>;

    async fn delete_all_in_index(
        &self,
        columns: &[&str],
        query: Query<impl KeyTuple>,
    ) -> Result<(), Self::Error>;

    /// Deletes all entries in the collection.
    fn truncate(&self) -> Result<(), Self::Error>;

    async fn set_entity<E: Entity + Send + Sync>(&self, entity: &E) -> Result<(), Self::Error> {
        self.set(entity.id(), entity).await
    }

    fn put_entity<E: Entity>(&self, entity: &E) -> Result<(), Self::Error> {
        self.put(entity.id(), entity)
    }
}

pub trait ReadWriteTransaction<'tx>: ReadTransaction<'tx> + WriteTransaction<'tx> {}

pub struct IndexSpec {
    pub keys: Vec<String>,
    pub unique: bool,
}

pub struct IndexSpecBuilder {
    keys: Vec<String>,
    unique: bool,
}

impl IndexSpec {
    /// Creates a new index with the name `name`. Note that the name must match the name of a field
    /// of the `Collection`'s type.
    pub fn builder() -> IndexSpecBuilder {
        IndexSpecBuilder {
            keys: vec![],
            unique: false,
        }
    }
}

impl IndexSpecBuilder {
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn add_column(mut self, column: impl Into<String>) -> Self {
        self.keys.push(column.into());
        self
    }
}

impl IndexSpecBuilder {
    pub fn build(self) -> IndexSpec {
        IndexSpec {
            keys: self.keys,
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

pub trait KeyType: Send + Sync + Debug + PartialEq {
    fn to_raw_key(&self) -> RawKey;
}

macro_rules! to_raw_key(
    ($t:ty) => (
        impl KeyType for $t {
            #[inline]
            fn to_raw_key(&self) -> RawKey {
                RawKey::from(*self)
            }
        }
    )
);

macro_rules! to_raw_key_str(
    ($t:ty) => (
        impl KeyType for $t {
            #[inline]
            fn to_raw_key(&self) -> RawKey {
                RawKey::from(self.to_string())
            }
        }
    )
);

#[derive(Debug, PartialEq)]
pub enum RawKey {
    Integer(i64),
    Real(f64),
    Text(String),
}

impl Serialize for RawKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            RawKey::Integer(ref i) => serializer.serialize_i64(*i),
            RawKey::Real(ref f) => serializer.serialize_f64(*f),
            RawKey::Text(ref s) => serializer.serialize_str(s),
        }
    }
}

impl From<i32> for RawKey {
    fn from(value: i32) -> Self {
        Self::Integer(value.into())
    }
}

impl From<i64> for RawKey {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<f32> for RawKey {
    fn from(value: f32) -> Self {
        Self::Real(value.into())
    }
}

impl From<f64> for RawKey {
    fn from(value: f64) -> Self {
        Self::Real(value)
    }
}

impl From<String> for RawKey {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&String> for RawKey {
    fn from(value: &String) -> Self {
        Self::Text(value.to_owned())
    }
}

impl<'a> From<&str> for RawKey {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl KeyType for str {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.to_string())
    }
}

impl KeyType for String {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Text(self.clone())
    }
}

impl KeyType for u32 {
    fn to_raw_key(&self) -> RawKey {
        RawKey::Integer(*self as i64)
    }
}

impl<T: KeyType> KeyType for &T {
    fn to_raw_key(&self) -> RawKey {
        (*self).to_raw_key()
    }
}

to_raw_key!(i32);
to_raw_key!(i64);
to_raw_key!(f32);
to_raw_key!(f64);
to_raw_key!(&str);

#[cfg(feature = "chrono")]
mod chrono {
    use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};

    use super::{KeyType, RawKey};

    /// N.B: DateTime<Local> and DateTime<FixedOffset> are not supported as keys, since these get
    /// encoded with their timezone, i.e. "2022-09-15T16:10:00+07:00".
    /// If you'd want to fetch an object for that key you'd have to create a DateTime with the
    /// exact same timezone, otherwise your query wouldn't match.
    impl KeyType for DateTime<Utc> {
        fn to_raw_key(&self) -> RawKey {
            // We're using a custom format here, which boils down to RFC 3339, however converting
            // DateTime::<Utc>::MAX_UTC to a string with to_rfc3339 would result in '+262142-12-31T23:59:59Z'.
            // In that example the leading + sign indicates that the year is outside the usual
            // range (0000-9999), which is a feature of the extended format specified by ISO 8601,
            // upon which RFC 3339 is based. SQLite however does not parse that string properly.

            let formatted = format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                self.year(),
                self.month(),
                self.day(),
                self.hour(),
                self.minute(),
                self.second()
            );
            RawKey::Text(formatted)
        }
    }

    impl KeyType for NaiveDate {
        fn to_raw_key(&self) -> RawKey {
            RawKey::Text(self.format("%Y-%m-%d").to_string())
        }
    }
}

#[cfg(feature = "uuid")]
mod uuid {
    use uuid::Uuid;

    use super::{KeyType, RawKey};

    impl KeyType for Uuid {
        fn to_raw_key(&self) -> RawKey {
            RawKey::Text(self.to_string())
        }
    }
}

#[cfg(feature = "jid")]
mod jid {
    use jid::{BareJid, FullJid, Jid};

    use super::{KeyType, RawKey};

    to_raw_key_str!(BareJid);
    to_raw_key_str!(FullJid);
    to_raw_key_str!(Jid);
}

pub enum Query<T> {
    All,
    Range { start: Bound<T>, end: Bound<T> },
    Only(T),
}

impl<T: KeyTuple + Clone> Query<T> {
    pub fn from_range<B: RangeBounds<T>>(range: B) -> Self {
        Self::Range {
            start: range.start_bound().cloned(),
            end: range.end_bound().cloned(),
        }
    }
}

pub trait KeyTuple: Send + Sync + Debug {
    fn to_raw_keys(&self) -> Vec<RawKey>;
}

impl<A: KeyType, B: KeyType> KeyTuple for (A, B) {
    fn to_raw_keys(&self) -> Vec<RawKey> {
        vec![self.0.to_raw_key(), self.1.to_raw_key()]
    }
}

impl<A: KeyType, B: KeyType, C: KeyType> KeyTuple for (A, B, C) {
    fn to_raw_keys(&self) -> Vec<RawKey> {
        vec![
            self.0.to_raw_key(),
            self.1.to_raw_key(),
            self.2.to_raw_key(),
        ]
    }
}

impl<A: KeyType, B: KeyType, C: KeyType, D: KeyType> KeyTuple for (A, B, C, D) {
    fn to_raw_keys(&self) -> Vec<RawKey> {
        vec![
            self.0.to_raw_key(),
            self.1.to_raw_key(),
            self.2.to_raw_key(),
            self.3.to_raw_key(),
        ]
    }
}

impl<T: KeyType> KeyTuple for T {
    fn to_raw_keys(&self) -> Vec<RawKey> {
        vec![self.to_raw_key()]
    }
}

impl KeyTuple for str {
    fn to_raw_keys(&self) -> Vec<RawKey> {
        vec![RawKey::Text(self.to_string())]
    }
}
