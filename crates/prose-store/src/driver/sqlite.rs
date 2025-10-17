use std::collections::{HashMap, HashSet};
use std::iter::zip;
use std::marker::PhantomData;
use std::ops::Bound;
use std::path::PathBuf;
use std::sync::{Arc, PoisonError};

use async_trait::async_trait;
use deadpool_sqlite::{
    Config, CreatePoolError, Hook, InteractError, Manager, Pool, PoolError, Runtime,
};
use parking_lot::RwLock;
use rusqlite::trace::{TraceEvent, TraceEventCodes};
use rusqlite::{
    params, params_from_iter, DropBehavior, OptionalExtension, ToSql, TransactionBehavior,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::{Semaphore, SemaphorePermit};
use tracing::debug;

use prose_wasm_utils::SendUnlessWasm;

use crate::driver::{ReadMode, ReadOnly, ReadWrite, WriteMode};
use crate::prelude::Error::NotMemberOfTransaction;
use crate::{
    Collection, Database, IndexSpec, IndexedCollection, KeyTuple, KeyType, Query, QueryDirection,
    RawKey, ReadTransaction, ReadableCollection, StoreError, Transaction, UpgradeTransaction,
    VersionChangeEvent, WritableCollection, WriteTransaction,
};

use super::Driver;

const SETTINGS_TABLE: &str = "__store_settings";

pub struct SqliteDriver {
    path: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    CreatePool(#[from] CreatePoolError),

    #[error(transparent)]
    Pool(#[from] PoolError),

    #[error("Collection {collection} does not exist")]
    UnknownCollection { collection: String },

    #[error("Index {index} does not exist on collection {collection}")]
    UnknownIndex { collection: String, index: String },

    #[error("Index {index} on collection {collection} is invalid.")]
    InvalidIndex { collection: String, index: String },

    #[error("Collection {collection} is not a member of the current transaction")]
    NotMemberOfTransaction { collection: String },

    #[error("{0}")]
    Interact(String),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    JSON(#[from] serde_json::Error),

    #[error(transparent)]
    Acquire(#[from] tokio::sync::AcquireError),

    #[error("{0}")]
    Poison(String),
}

impl StoreError for Error {}

impl From<InteractError> for Error {
    fn from(value: InteractError) -> Self {
        Self::Interact(value.to_string())
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
        Self::Poison(value.to_string())
    }
}

impl SqliteDriver {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        SqliteDriver { path: path.into() }
    }
}

#[async_trait]
impl Driver for SqliteDriver {
    type Error = Error;

    type UpgradeTransaction<'db> = SqliteTransaction<'db, Upgrade>;
    type Database = SqliteDB;

    async fn open<F>(self, version: u32, update_handler: F) -> Result<Self::Database, Self::Error>
    where
        F: Fn(&VersionChangeEvent<Self::UpgradeTransaction<'_>>) -> Result<(), Self::Error>
            + Send
            + 'static,
    {
        let pool = Config::new(self.path)
            .builder(Runtime::Tokio1)
            .map_err(CreatePoolError::Config)?
            .post_create(Hook::sync_fn(|obj, _| {
                let conn = obj.lock().unwrap();

                conn.trace_v2(
                    TraceEventCodes::all(),
                    Some(|event| {
                        if let TraceEvent::Stmt(s, _) = event {
                            debug!("{}", s.sql());
                        }
                    }),
                );

                conn.execute_batch(
                    r#"
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = normal;
                PRAGMA journal_size_limit = 6144000;
                "#,
                )
                .expect("Failed to configure DB connection");

                Ok(())
            }))
            .build()
            .map_err(CreatePoolError::Build)?;

        let obj = pool.get().await?;

        let (current_version, description) = {
            let sql_conn = obj.lock()?;
            sql_conn.execute_batch(&format!(
                r#"
            CREATE TABLE IF NOT EXISTS "{}" (
                `key` TEXT PRIMARY KEY,
                `value` BLOB NOT NULL
            )"#,
                SETTINGS_TABLE
            ))?;

            let current_version = sql_conn
                .query_row(
                    &format!(
                        r#"SELECT "value" FROM "{}" WHERE "key" = "version""#,
                        SETTINGS_TABLE
                    ),
                    [],
                    |row| row.get::<_, u32>(0),
                )
                .optional()?
                .unwrap_or(0);

            (current_version, sql_conn.database_description()?)
        };

        let db = SqliteDB {
            pool,
            write_lock: Semaphore::new(1),
            description: description.clone(),
        };

        if current_version != version {
            let tx = SqliteTransaction::new(
                description.all_table_names().into_iter().collect(),
                db.pool.get().await?,
                description,
                Some(TransactionBehavior::Immediate),
                DropBehavior::Commit,
                None,
            )?;

            let tx = {
                let event = VersionChangeEvent {
                    tx,
                    old_version: current_version,
                    new_version: version,
                    phantom: Default::default(),
                };

                update_handler(&event)?;
                event.tx
            };

            let sql_conn = tx.obj.lock()?;
            sql_conn.execute(
                &format!(
                    r#"INSERT OR REPLACE INTO "{}" ("key", "value") VALUES ('version', ?)"#,
                    SETTINGS_TABLE
                ),
                params![version],
            )?;

            return Ok(db);
        }

        Ok(db)
    }
}

pub struct SqliteDB {
    pool: Pool,
    write_lock: Semaphore,
    description: DatabaseDescription,
}

#[async_trait]
impl Database for SqliteDB {
    type Error = Error;

    type ReadTransaction<'db> = SqliteTransaction<'db, ReadOnly>;
    type ReadWriteTransaction<'db> = SqliteTransaction<'db, ReadWrite>;

    async fn collection_names(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.description.all_table_names())
    }

    async fn transaction_for_reading(
        &self,
        collections: &[&str],
    ) -> Result<Self::ReadTransaction<'_>, Self::Error> {
        for collection in collections {
            if !self.description.contains_table(collection) {
                return Err(Error::UnknownCollection {
                    collection: collection.to_string(),
                });
            }
        }

        SqliteTransaction::new(
            collections.iter().map(ToString::to_string).collect(),
            self.pool.get().await?,
            self.description.clone(),
            None,
            DropBehavior::Ignore,
            None,
        )
    }

    async fn transaction_for_reading_and_writing(
        &self,
        collections: &[&str],
    ) -> Result<Self::ReadWriteTransaction<'_>, Self::Error> {
        for collection in collections {
            if !self.description.contains_table(collection) {
                return Err(Error::UnknownCollection {
                    collection: collection.to_string(),
                });
            }
        }

        SqliteTransaction::new(
            collections.iter().map(ToString::to_string).collect(),
            self.pool.get().await?,
            self.description.clone(),
            Some(TransactionBehavior::Immediate),
            DropBehavior::Commit,
            Some(self.write_lock.acquire().await?),
        )
    }
}

impl SqliteDB {
    #[cfg(feature = "test")]
    pub async fn describe_table(&self, name: &str) -> Result<String, Error> {
        let obj = self.pool.get().await?;
        let conn = obj.lock()?;
        let sql = conn.query_row(
            "SELECT sql FROM sqlite_schema WHERE name = ?",
            params![name],
            |row| row.get(0),
        )?;
        Ok(sql)
    }
}

pub struct Upgrade;

pub struct SqliteTransaction<'db, Mode> {
    member_collections: HashSet<String>,
    obj: Arc<deadpool::managed::Object<Manager>>,
    drop_behavior: DropBehavior,
    description: DatabaseDescription,
    _write_permit: Option<SemaphorePermit<'db>>,
    phantom: PhantomData<&'db Mode>,
}

impl<'db> UpgradeTransaction<'db> for SqliteTransaction<'db, Upgrade> {
    type Error = Error;
    type ReadWriteTransaction<'tx>
        = SqliteTransaction<'tx, ReadWrite>
    where
        Self: 'tx;

    fn collection_names(&self) -> Result<Vec<String>, Self::Error> {
        Ok(self.description.all_table_names())
    }

    fn create_collection(
        &self,
        name: &str,
    ) -> Result<
        <Self::ReadWriteTransaction<'_> as WriteTransaction<'_>>::WritableCollection<'_>,
        Self::Error,
    > {
        let conn = self.obj.lock()?;
        conn.execute(
            &format!(
                r#"
        CREATE TABLE "{}" (
            `key` TEXT PRIMARY KEY,
            `data` TEXT
        )"#,
                name
            ),
            params![],
        )?;

        self.description.add_table(name);

        Ok(SqliteCollection::new(
            name.to_string(),
            self.obj.clone(),
            self.description.clone(),
        ))
    }

    fn delete_collection(&self, name: &str) -> Result<(), Self::Error> {
        self.obj
            .lock()?
            .execute(&format!(r#"DROP TABLE "{}""#, name), params![])?;
        self.description.remove_table(name);
        Ok(())
    }
}

impl<'db, Mode> SqliteTransaction<'db, Mode> {
    fn new(
        member_collections: HashSet<String>,
        obj: deadpool::managed::Object<Manager>,
        description: DatabaseDescription,
        behavior: Option<TransactionBehavior>,
        drop_behavior: DropBehavior,
        write_permit: Option<SemaphorePermit<'db>>,
    ) -> Result<Self, Error> {
        if let Some(behavior) = behavior {
            let conn = obj.lock()?;

            let query = match behavior {
                TransactionBehavior::Deferred => "BEGIN DEFERRED",
                TransactionBehavior::Immediate => "BEGIN IMMEDIATE",
                TransactionBehavior::Exclusive => "BEGIN EXCLUSIVE",
                _ => unreachable!("Unexpected TransactionBehavior"),
            };
            conn.execute_batch(query)?;
        }

        Ok(SqliteTransaction {
            member_collections,
            obj: Arc::new(obj),
            drop_behavior,
            description,
            _write_permit: write_permit,
            phantom: Default::default(),
        })
    }

    fn commit_(self) -> Result<(), Error> {
        self.obj.lock()?.execute_batch("COMMIT")?;
        Ok(())
    }

    fn rollback_(self) -> Result<(), Error> {
        self.obj.lock()?.execute_batch("ROLLBACK")?;
        Ok(())
    }
}

impl<'db, Mode> Drop for SqliteTransaction<'db, Mode> {
    fn drop(&mut self) {
        let Ok(conn) = self.obj.lock() else {
            return;
        };

        match self.drop_behavior {
            DropBehavior::Commit => {
                if conn.execute_batch("COMMIT") != Ok(()) {
                    _ = conn.execute_batch("ROLLBACK");
                }
            }
            DropBehavior::Rollback => _ = conn.execute_batch("ROLLBACK"),
            DropBehavior::Ignore => (),
            DropBehavior::Panic => panic!("Transaction dropped unexpectedly."),
            _ => unreachable!("Unexpected DropBehavior"),
        }
    }
}

impl<'db, Mode: Send + Sync> Transaction<'db> for SqliteTransaction<'db, Mode> {
    type Error = Error;
}

impl<'db, Mode: Send + Sync> ReadTransaction<'db> for SqliteTransaction<'db, Mode>
where
    Mode: ReadMode,
{
    type ReadableCollection<'tx>
        = SqliteCollection<'tx, ReadOnly>
    where
        Self: 'tx;

    fn readable_collection(&self, name: &str) -> Result<Self::ReadableCollection<'_>, Self::Error> {
        if !self.member_collections.contains(name) {
            return Err(NotMemberOfTransaction {
                collection: name.to_string(),
            });
        }
        Ok(SqliteCollection::new(
            name.to_string(),
            self.obj.clone(),
            self.description.clone(),
        ))
    }
}

#[async_trait]
impl<'db, Mode: Send + Sync> WriteTransaction<'db> for SqliteTransaction<'db, Mode>
where
    Mode: WriteMode + Sync,
{
    type WritableCollection<'tx>
        = SqliteCollection<'tx, ReadWrite>
    where
        Self: 'tx;

    fn writeable_collection(
        &self,
        name: &str,
    ) -> Result<Self::WritableCollection<'_>, Self::Error> {
        if !self.member_collections.contains(name) {
            return Err(NotMemberOfTransaction {
                collection: name.to_string(),
            });
        }
        Ok(SqliteCollection::new(
            name.to_string(),
            self.obj.clone(),
            self.description.clone(),
        ))
    }

    async fn commit(self) -> Result<(), Self::Error> {
        self.commit_()
    }

    async fn rollback(self) -> Result<(), Self::Error> {
        self.rollback_()
    }
}

pub struct SqliteCollection<'tx, Mode> {
    name: String,
    qualified_columns: Vec<String>,
    obj: Arc<deadpool::managed::Object<Manager>>,
    description: DatabaseDescription,
    phantom: PhantomData<&'tx Mode>,
}

impl<'tx, Mode> SqliteCollection<'tx, Mode> {
    fn new(
        name: String,
        obj: Arc<deadpool::managed::Object<Manager>>,
        description: DatabaseDescription,
    ) -> Self {
        Self {
            name,
            qualified_columns: vec![r#""key""#.to_string()],
            obj,
            description,
            phantom: Default::default(),
        }
    }

    fn new_index(
        name: String,
        columns: &[&str],
        obj: Arc<deadpool::managed::Object<Manager>>,
        description: DatabaseDescription,
    ) -> Self {
        Self {
            name,
            qualified_columns: columns
                .iter()
                .map(|column| format!(r#"json_extract("data", '$.{}')"#, column))
                .collect(),
            obj,
            description,
            phantom: Default::default(),
        }
    }

    fn qualified_key_columns(&self) -> &[String] {
        self.qualified_columns.as_slice()
    }

    #[cfg(feature = "test")]
    pub fn explain_query_plan<Key: KeyTuple + Send>(
        &self,
        query: Query<Key>,
    ) -> Result<String, Error> {
        let conn = self.obj.lock()?;

        let (sql, params) = query.into_sql(
            &self.name,
            &self.qualified_key_columns(),
            QueryDirection::Forward,
            None,
        );

        Ok(conn.query_row(
            &format!("EXPLAIN QUERY PLAN {sql}"),
            params_from_iter(params),
            |row| row.get::<_, String>(3),
        )?)
    }
}

impl<'tx, Mode: Send + Sync> Collection<'tx> for SqliteCollection<'tx, Mode> {
    type Error = Error;
}

impl<'tx, Mode: Send> IndexedCollection<'tx> for SqliteCollection<'tx, Mode>
where
    Mode: ReadMode + Sync,
{
    type Index<'coll>
        = SqliteCollection<'coll, Mode>
    where
        Self: 'coll;

    fn index(&self, columns: &[&str]) -> Result<Self::Index<'_>, Self::Error> {
        let index_name = columns.join("_");

        if !self
            .description
            .table_contains_index(&self.name, &index_name)
        {
            return Err(Error::UnknownIndex {
                collection: self.name.to_string(),
                index: index_name,
            });
        }

        Ok(SqliteCollection::new_index(
            self.name.clone(),
            columns,
            self.obj.clone(),
            self.description.clone(),
        ))
    }
}

#[async_trait]
impl<'tx, Mode: Send + Sync> ReadableCollection<'tx> for SqliteCollection<'tx, Mode>
where
    Mode: ReadMode + Sync,
{
    async fn get<K: KeyTuple + ?Sized, V: DeserializeOwned>(
        &self,
        key: &K,
    ) -> Result<Option<V>, Self::Error> {
        let values = key.to_raw_keys();
        let columns = self.qualified_key_columns();

        assert_eq!(
            values.len(),
            columns.len(),
            "The number of tuple fields should match the number of columns in the index"
        );

        let sql = format!(
            r#"SELECT "data" FROM "{}" WHERE {} LIMIT 1"#,
            self.name,
            columns
                .iter()
                .map(|column| format!("{column} = ?"))
                .collect::<Vec<_>>()
                .join(" AND ")
        );

        let conn = self.obj.lock()?;
        let mut statement = conn.prepare(&sql)?;
        let data = statement
            .query_row(params_from_iter(values), |row| row.get::<_, String>(0))
            .optional()?;
        let result = data.map(|data| serde_json::from_str(&data)).transpose()?;

        Ok(result)
    }

    async fn contains_key<K: KeyTuple + ?Sized>(&self, key: &K) -> Result<bool, Self::Error> {
        let values = key.to_raw_keys();
        let columns = self.qualified_key_columns();

        assert_eq!(
            values.len(),
            columns.len(),
            "The number of tuple fields should match the number of columns in the index"
        );

        let conn = self.obj.lock()?;
        let mut statement = conn.prepare(&format!(
            "SELECT EXISTS(SELECT 1 FROM '{}' WHERE {})",
            self.name,
            columns
                .iter()
                .map(|column| format!("{column} = ?"))
                .collect::<Vec<_>>()
                .join(" AND ")
        ))?;
        Ok(statement.query_row(params_from_iter(values), |row| row.get(0))?)
    }

    async fn all_keys(&self) -> Result<Vec<String>, Self::Error> {
        let columns = self.qualified_key_columns();
        assert_eq!(
            columns.len(),
            1,
            "all_keys is not supported for multi-column indexes."
        );

        let conn = self.obj.lock()?;
        let mut statement = conn.prepare(&format!("SELECT {} FROM '{}'", columns[0], self.name))?;
        let result = statement
            .query_map(params![], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    async fn get_all<Value: DeserializeOwned + Send>(
        &self,
        query: Query<impl KeyTuple>,
        direction: QueryDirection,
        limit: Option<usize>,
    ) -> Result<Vec<(String, Value)>, Self::Error> {
        self.fold_into_vec(
            query,
            |key, value| Some((key, value)),
            direction,
            limit,
            true,
        )
        .await
    }

    async fn get_all_filtered<Value: DeserializeOwned + Send, T: Send>(
        &self,
        query: Query<impl KeyTuple>,
        direction: QueryDirection,
        limit: Option<usize>,
        filter: impl FnMut(String, Value) -> Option<T> + SendUnlessWasm,
    ) -> Result<Vec<T>, Self::Error> {
        self.fold_into_vec(query, filter, direction, limit, false)
            .await
    }

    async fn fold<Value: DeserializeOwned + Send, T: Send>(
        &self,
        query: Query<impl KeyTuple>,
        init: T,
        mut f: impl FnMut(T, (String, Value)) -> T + SendUnlessWasm,
    ) -> Result<T, Self::Error> {
        self._fold(
            query,
            QueryDirection::default(),
            None,
            init,
            |result, args, _| f(result, args),
        )
        .await
    }
}

impl<'tx, Mode> SqliteCollection<'tx, Mode>
where
    Mode: ReadMode + Sync,
{
    async fn fold_into_vec<Key: KeyTuple, Value: DeserializeOwned + Send, T: Send>(
        &self,
        query: Query<Key>,
        mut filter: impl FnMut(String, Value) -> Option<T> + Send,
        direction: QueryDirection,
        limit: Option<usize>,
        limit_query: bool,
    ) -> Result<Vec<T>, Error> {
        if limit == Some(0) {
            return Ok(vec![]);
        }

        let db_limit = if limit_query { limit } else { None };
        let limit = limit.unwrap_or(usize::MAX);

        self._fold(
            query,
            direction,
            db_limit,
            vec![],
            |mut result, (key, value), stop| {
                if let Some(transformed_value) = filter(key, value) {
                    result.push(transformed_value);
                }
                if result.len() == limit {
                    *stop = true;
                }
                result
            },
        )
        .await
    }

    async fn _fold<Key: KeyTuple, Value: DeserializeOwned + Send, T: Send>(
        &self,
        query: Query<Key>,
        direction: QueryDirection,
        limit: Option<usize>,
        init: T,
        mut f: impl FnMut(T, (String, Value), &mut bool) -> T + SendUnlessWasm,
    ) -> Result<T, Error> {
        let conn = self.obj.lock()?;
        let (sql, params) =
            query.into_sql(&self.name, self.qualified_key_columns(), direction, limit);
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(params), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut last_value = init;

        for row in rows {
            let (key, data) = row?;
            let value = serde_json::from_str(&data)?;
            let mut stop = false;
            last_value = f(last_value, (key, value), &mut stop);
            if stop {
                break;
            }
        }

        Ok(last_value)
    }
}

#[async_trait]
impl<'tx> WritableCollection<'tx> for SqliteCollection<'tx, ReadWrite> {
    fn add_index(&self, idx: IndexSpec) -> Result<(), Self::Error> {
        let conn = self.obj.lock()?;
        let index_type = if idx.unique { "UNIQUE INDEX" } else { "INDEX" };
        let index_name = idx.keys.join("_");
        let columns = idx
            .keys
            .into_iter()
            .map(|key| format!(r#"json_extract("data", '$.{key}')"#))
            .collect::<Vec<_>>()
            .join(", ");

        let sql = &format!(
            "CREATE {index_type} 'prose_{table_name}_{index_name}_idx' ON '{table_name}'({columns})",
            table_name = self.name,
        );
        let mut statement = conn.prepare(&sql)?;
        statement.execute(params![])?;
        self.description.add_index(&self.name, &index_name);
        Ok(())
    }

    async fn set<K: KeyType + ?Sized, V: Serialize + ?Sized + Send + Sync>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        let conn = self.obj.lock()?;
        let mut statement = conn.prepare(&format!(
            r#"INSERT INTO "{}" ("key", "data") VALUES (?, ?)"#,
            self.name
        ))?;
        statement.execute(params![key.to_raw_key(), &serde_json::to_string(value)?])?;
        Ok(())
    }

    fn put<K: KeyType + ?Sized, V: Serialize>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        let conn = self.obj.lock()?;
        let mut statement = conn.prepare(&format!(
            r#"INSERT OR REPLACE INTO "{}" ("key", "data") VALUES (?, ?)"#,
            self.name
        ))?;
        statement.execute(params![key.to_raw_key(), &serde_json::to_string(value)?])?;
        Ok(())
    }

    async fn delete<K: KeyTuple + ?Sized>(&self, key: &K) -> Result<(), Self::Error> {
        let values = key.to_raw_keys();
        let columns = self.qualified_key_columns();

        assert_eq!(
            values.len(),
            columns.len(),
            "The number of tuple fields should match the number of columns in the index"
        );

        let sql = format!(
            r#"DELETE FROM "{}" WHERE {}"#,
            self.name,
            columns
                .iter()
                .map(|column| format!("{column} = ?"))
                .collect::<Vec<_>>()
                .join(" AND ")
        );

        let conn = self.obj.lock()?;
        let mut statement = conn.prepare(&sql)?;
        statement.execute(params_from_iter(values))?;
        Ok(())
    }

    async fn delete_all_in_index(
        &self,
        columns: &[&str],
        query: Query<impl KeyTuple>,
    ) -> Result<(), Self::Error> {
        let conn = self.obj.lock()?;
        let qualified_columns = columns
            .iter()
            .map(|column| format!(r#"json_extract("data", '$.{}')"#, column))
            .collect::<Vec<_>>();

        let mut sql = format!("DELETE FROM '{}'", self.name);
        let params = match query.into_sql_predicate(&qualified_columns) {
            Some((predicate, params)) => {
                sql.push_str(&format!(" WHERE {predicate}"));
                params
            }
            None => vec![],
        };

        let mut statement = conn.prepare(&sql)?;
        statement.execute(params_from_iter(params))?;
        Ok(())
    }

    fn truncate(&self) -> Result<(), Self::Error> {
        let conn = self.obj.lock()?;
        let mut statement = conn.prepare(&format!(r#"DELETE FROM '{}'"#, self.name))?;
        statement.execute([])?;
        Ok(())
    }
}

#[derive(Clone)]
struct DatabaseDescription {
    tables_to_indexes_map: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl DatabaseDescription {
    fn contains_table(&self, table_name: &str) -> bool {
        self.tables_to_indexes_map.read().contains_key(table_name)
    }

    fn table_contains_index(&self, table_name: &str, index_name: &str) -> bool {
        let map = self.tables_to_indexes_map.read();
        let Some(indexes) = map.get(table_name) else {
            return false;
        };
        indexes.contains(index_name)
    }

    fn all_table_names(&self) -> Vec<String> {
        self.tables_to_indexes_map.read().keys().cloned().collect()
    }

    fn add_table(&self, table_name: &str) {
        self.tables_to_indexes_map
            .write()
            .entry(table_name.to_string())
            .or_insert(HashSet::new());
    }

    fn remove_table(&self, table_name: &str) {
        self.tables_to_indexes_map.write().remove_entry(table_name);
    }

    fn add_index(&self, table_name: &str, index_name: &str) {
        self.tables_to_indexes_map
            .write()
            .entry(table_name.to_string())
            .or_insert(HashSet::new())
            .insert(index_name.to_string());
    }
}

trait ConnectionExt {
    fn database_description(&self) -> Result<DatabaseDescription, Error>;
}

impl ConnectionExt for rusqlite::Connection {
    fn database_description(&self) -> Result<DatabaseDescription, Error> {
        let mut tables_to_indexes_map = HashMap::new();

        // Order the rows so that type=table comes before type=index…
        let mut statement = self.prepare(
            r#"SELECT "type", "name", "tbl_name" FROM sqlite_schema ORDER BY "type" DESC"#,
        )?;
        let mut rows = statement.query([])?;

        while let Some(row) = rows.next()? {
            let type_ = row.get::<_, String>(0)?;
            let name = row.get::<_, String>(1)?;

            match type_.as_str() {
                "table" => {
                    if name == SETTINGS_TABLE {
                        continue;
                    }
                    tables_to_indexes_map.entry(name).or_insert(HashSet::new());
                }
                "index" => {
                    if name.strip_prefix("prose_").is_none() {
                        continue;
                    }

                    let table_name = row.get::<_, String>(2)?;
                    let table_name_prefix = table_name.clone() + "_";

                    // e.g: prose_{table_name}_field_idx
                    if !name[6..].starts_with(&table_name_prefix) {
                        return Err(Error::InvalidIndex {
                            collection: table_name,
                            index: name,
                        });
                    }

                    let idx_name = &name[(6 + table_name_prefix.len())..name.len() - 4];

                    tables_to_indexes_map
                        .entry(table_name)
                        .or_insert(HashSet::new())
                        .insert(idx_name.to_string());
                }
                _ => (),
            }
        }

        Ok(DatabaseDescription {
            tables_to_indexes_map: Arc::new(RwLock::new(tables_to_indexes_map)),
        })
    }
}

impl<T: KeyTuple> Query<T> {
    fn into_sql(
        self,
        table: &str,
        columns: &[String],
        direction: QueryDirection,
        limit: Option<usize>,
    ) -> (String, Vec<RawKey>) {
        let order = match direction {
            QueryDirection::Forward => "ASC",
            QueryDirection::Backward => "DESC",
        };

        let mut sql = format!(r#"SELECT "key", "data" FROM "{table}""#);

        let params = match self.into_sql_predicate(columns) {
            Some((predicate, params)) => {
                sql.push_str(&format!(" WHERE {predicate}"));
                params
            }
            None => vec![],
        };

        sql.push_str(&format!(
            " ORDER BY {column_order}",
            column_order = columns
                .iter()
                .map(|column| format!(r#"{column} {order}"#))
                .collect::<Vec<_>>()
                .join(", ")
        ));

        if let Some(limit) = limit {
            sql.push_str(&format!(" LIMIT {limit}"))
        }

        (sql, params)
    }

    fn into_sql_predicate(self, columns: &[impl AsRef<str>]) -> Option<(String, Vec<RawKey>)> {
        let (predicates, params) = match self {
            Query::All => (vec![], vec![]),
            Query::Range { start, end } => {
                fn into_bounds<A: KeyTuple, B: KeyTuple>(
                    tuple1: A,
                    bound1: impl Fn(RawKey) -> Bound<RawKey>,
                    tuple2: B,
                    bound2: impl Fn(RawKey) -> Bound<RawKey>,
                ) -> (Vec<Bound<RawKey>>, Vec<Bound<RawKey>>) {
                    (
                        tuple1.to_raw_keys().into_iter().map(bound1).collect(),
                        tuple2.to_raw_keys().into_iter().map(bound2).collect(),
                    )
                }

                let (start, end) = match (start, end) {
                    (Bound::Included(start), Bound::Included(end)) => {
                        into_bounds(start, Bound::Included, end, Bound::Included)
                    }
                    (Bound::Included(start), Bound::Excluded(end)) => {
                        into_bounds(start, Bound::Included, end, Bound::Excluded)
                    }
                    (Bound::Included(start), Bound::Unbounded) => {
                        let start = start
                            .to_raw_keys()
                            .into_iter()
                            .map(Bound::Included)
                            .collect::<Vec<_>>();
                        let mut end = vec![];
                        end.resize_with(start.len(), || Bound::Unbounded);
                        (start, end)
                    }
                    (Bound::Excluded(start), Bound::Excluded(end)) => {
                        into_bounds(start, Bound::Excluded, end, Bound::Excluded)
                    }
                    (Bound::Excluded(start), Bound::Included(end)) => {
                        into_bounds(start, Bound::Excluded, end, Bound::Included)
                    }
                    (Bound::Excluded(start), Bound::Unbounded) => {
                        let start = start
                            .to_raw_keys()
                            .into_iter()
                            .map(Bound::Excluded)
                            .collect::<Vec<_>>();
                        let mut end = vec![];
                        end.resize_with(start.len(), || Bound::Unbounded);
                        (start, end)
                    }
                    (Bound::Unbounded, Bound::Unbounded) => (vec![], vec![]),
                    (Bound::Unbounded, Bound::Included(end)) => {
                        let end = end
                            .to_raw_keys()
                            .into_iter()
                            .map(Bound::Included)
                            .collect::<Vec<_>>();
                        let mut start = vec![];
                        start.resize_with(end.len(), || Bound::Unbounded);
                        (start, end)
                    }
                    (Bound::Unbounded, Bound::Excluded(end)) => {
                        let end = end
                            .to_raw_keys()
                            .into_iter()
                            .map(Bound::Excluded)
                            .collect::<Vec<_>>();
                        let mut start = vec![];
                        start.resize_with(end.len(), || Bound::Unbounded);
                        (start, end)
                    }
                };

                assert_eq!(
                    start.len(),
                    end.len(),
                    "Both bounds should have the same number of tuple fields."
                );
                assert!(
                    start.is_empty() || start.len() == columns.len(),
                    "The number of tuple fields should match the number of columns in the index"
                );

                zip(zip(start, end), columns).into_iter().fold(
                    (vec![], vec![]),
                    |(mut predicates_vec, mut params_vec), ((start, end), column)| {
                        let (predicate, params) =
                            Query::Range { start, end }.into_where_clause(column.as_ref());

                        if !predicate.is_empty() {
                            predicates_vec.push(predicate);
                            params_vec.extend(params);
                        }

                        (predicates_vec, params_vec)
                    },
                )
            }
            Query::Only(values) => zip(values.to_raw_keys(), columns).into_iter().fold(
                (vec![], vec![]),
                |(mut predicates_vec, mut params_vec), (value, column)| {
                    let (predicate, params) = Query::Only(value).into_where_clause(column.as_ref());
                    predicates_vec.push(predicate);
                    params_vec.extend(params);
                    (predicates_vec, params_vec)
                },
            ),
        };

        if predicates.is_empty() {
            return None;
        };

        Some((predicates.join(" AND "), params))
    }
}

impl Query<RawKey> {
    fn into_where_clause(self, column: &str) -> (String, Vec<RawKey>) {
        let (predicate, params) = match self {
            Query::All => ("".to_string(), vec![]),

            Query::Range {
                start: Bound::Included(start),
                end: Bound::Included(end),
            } => (
                format!("({column} >= ? AND {column} <= ?)"),
                vec![start, end],
            ),
            Query::Range {
                start: Bound::Included(start),
                end: Bound::Excluded(end),
            } if start == end => (format!("{column} = ?"), vec![start]),
            Query::Range {
                start: Bound::Included(start),
                end: Bound::Excluded(end),
            } => (
                format!("({column} >= ? AND {column} < ?)"),
                vec![start, end],
            ),
            Query::Range {
                start: Bound::Included(start),
                end: Bound::Unbounded,
            } => (format!("{column} >= ?"), vec![start]),
            Query::Range {
                start: Bound::Excluded(start),
                end: Bound::Included(end),
            } if start == end => (format!("{column} = ?"), vec![start]),
            Query::Range {
                start: Bound::Excluded(start),
                end: Bound::Included(end),
            } => (
                format!("({column} > ? AND {column} <= ?)"),
                vec![start, end],
            ),
            Query::Range {
                start: Bound::Excluded(start),
                end: Bound::Excluded(end),
            } if start == end => (format!("{column} != ?"), vec![start]),
            Query::Range {
                start: Bound::Excluded(start),
                end: Bound::Excluded(end),
            } => (format!("({column} > ? AND {column} < ?)"), vec![start, end]),
            Query::Range {
                start: Bound::Excluded(start),
                end: Bound::Unbounded,
            } => (format!("{column} > ?"), vec![start]),

            Query::Range {
                start: Bound::Unbounded,
                end: Bound::Included(end),
            } => (format!("{column} <= ?"), vec![end]),
            Query::Range {
                start: Bound::Unbounded,
                end: Bound::Excluded(end),
            } => (format!("{column} < ?"), vec![end]),
            Query::Range {
                start: Bound::Unbounded,
                end: Bound::Unbounded,
            } => ("".to_string(), vec![]),

            Query::Only(value) => (format!("{column} = ?"), vec![value]),
        };

        (predicate, params)
    }
}

impl ToSql for RawKey {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        use rusqlite::types::{ToSqlOutput, ValueRef};

        match self {
            RawKey::Integer(value) => Ok(ToSqlOutput::Borrowed(ValueRef::Integer(value.clone()))),
            RawKey::Real(value) => Ok(ToSqlOutput::Borrowed(ValueRef::Real(value.clone()))),
            RawKey::Text(value) => Ok(ToSqlOutput::Borrowed(ValueRef::Text(value.as_bytes()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_to_where_clause() {
        let table = "table";
        let columns = &["account".to_string(), "user_id".to_string()];

        let query = Query::from_range(("a@prose.org", 2)..("a@prose.org", 3));
        assert_eq!(
            r#"SELECT "key", "data" FROM "table" WHERE account = ? AND (user_id >= ? AND user_id < ?) ORDER BY account ASC, user_id ASC"#,
            &query.into_sql(table, columns, Default::default(), None).0
        );

        let query = Query::from_range(("a@prose.org", 2)..("a@prose.org", 2));
        assert_eq!(
            r#"SELECT "key", "data" FROM "table" WHERE account = ? AND user_id = ? ORDER BY account ASC, user_id ASC"#,
            &query.into_sql(table, columns, Default::default(), None).0
        );

        let query = Query::from_range(("a@prose.org", 2)..("b@prose.org", 3));
        assert_eq!(
            r#"SELECT "key", "data" FROM "table" WHERE (account >= ? AND account < ?) AND (user_id >= ? AND user_id < ?) ORDER BY account ASC, user_id ASC"#,
            &query.into_sql(table, columns, Default::default(), None).0
        );

        let query = Query::from_range(..=("b@prose.org", 3));
        assert_eq!(
            r#"SELECT "key", "data" FROM "table" WHERE account <= ? AND user_id <= ? ORDER BY account ASC, user_id ASC"#,
            &query.into_sql(table, columns, Default::default(), None).0
        );

        let query = Query::<(&str, u32)>::from_range(..);
        assert_eq!(
            r#"SELECT "key", "data" FROM "table" ORDER BY account ASC, user_id ASC"#,
            &query.into_sql(table, columns, Default::default(), None).0
        );

        let query = Query::Range {
            start: Bound::Excluded(("a@prose.org", 2)),
            end: Bound::Excluded(("a@prose.org", 4)),
        };
        assert_eq!(
            r#"SELECT "key", "data" FROM "table" WHERE account != ? AND (user_id > ? AND user_id < ?) ORDER BY account ASC, user_id ASC"#,
            &query.into_sql(table, columns, Default::default(), None).0
        );
    }
}
