use super::Driver;
use crate::driver::{ReadMode, ReadOnly, ReadWrite, WriteMode};
use crate::prelude::Error::NotMemberOfTransaction;
use crate::{
    Collection, Database, IndexSpec, IndexedCollection, KeyType, Query, QueryDirection, RawKey,
    ReadTransaction, ReadableCollection, StoreError, Transaction, UpgradeTransaction,
    VersionChangeEvent, WritableCollection, WriteTransaction,
};
use async_trait::async_trait;
use deadpool_sqlite::{
    Config, CreatePoolError, Hook, InteractError, Manager, Pool, PoolError, Runtime,
};
use parking_lot::RwLock;
use prose_wasm_utils::SendUnlessWasm;
use rusqlite::{params, DropBehavior, OptionalExtension, ToSql, TransactionBehavior};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::ops::Bound;
use std::path::PathBuf;
use std::sync::{Arc, PoisonError};
use tracing::debug;

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

    #[error("Collection {collection} is not a member of the current transaction")]
    NotMemberOfTransaction { collection: String },

    #[error("{0}")]
    Interact(String),

    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),

    #[error(transparent)]
    JSON(#[from] serde_json::Error),

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
                let mut conn = obj.lock().unwrap();

                conn.trace(Some(|query| {
                    debug!("{}", query);
                }));

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
                "key" TEXT PRIMARY KEY,
                "value" BLOB NOT NULL
            )"#,
                SETTINGS_TABLE
            ))?;

            let current_version = sql_conn
                .query_row(
                    &format!(
                        r#"SELECT `value` FROM "{}" WHERE `key` = "version""#,
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
            description: description.clone(),
        };

        if current_version != version {
            let tx = SqliteTransaction::new(
                description.all_table_names().into_iter().collect(),
                db.pool.get().await?,
                description,
                Some(TransactionBehavior::Immediate),
                DropBehavior::Commit,
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
                    r#"INSERT OR REPLACE INTO "{}" (`key`, `value`) VALUES ('version', ?)"#,
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
    phantom: PhantomData<&'db Mode>,
}

impl<'db> UpgradeTransaction<'db> for SqliteTransaction<'db, Upgrade> {
    type Error = Error;
    type ReadWriteTransaction<'tx> = SqliteTransaction<'tx, ReadWrite> where Self: 'tx;

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
    ) -> Result<Self, Error> {
        if let Some(behavior) = behavior {
            let conn = obj.lock()?;

            let query = match behavior {
                TransactionBehavior::Deferred => "BEGIN DEFERRED",
                TransactionBehavior::Immediate => "BEGIN IMMEDIATE",
                TransactionBehavior::Exclusive => "BEGIN EXCLUSIVE",
                _ => unreachable!(),
            };
            conn.execute_batch(query)?;
        }

        Ok(SqliteTransaction {
            member_collections,
            obj: Arc::new(obj),
            drop_behavior,
            description,
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
            _ => unreachable!(),
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
    type ReadableCollection<'tx> = SqliteCollection<'tx, ReadOnly> where Self: 'tx;

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
    type WritableCollection<'tx> = SqliteCollection<'tx, ReadWrite> where Self: 'tx;

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
    is_index: bool,
    name: String,
    key_column: String,
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
            is_index: false,
            name,
            key_column: "key".to_string(),
            obj,
            description,
            phantom: Default::default(),
        }
    }

    fn qualified_key_column(&self) -> Cow<String> {
        if self.is_index {
            Cow::Owned(format!("json_extract(`data`, '$.{}')", self.key_column))
        } else {
            Cow::Borrowed(&self.key_column)
        }
    }

    #[cfg(feature = "test")]
    pub fn explain_query_plan<Key: KeyType + Send>(
        &self,
        query: Query<Key>,
    ) -> Result<String, Error> {
        let conn = self.obj.lock()?;

        let (sql, params) = query.to_sql(
            &self.name,
            &self.qualified_key_column(),
            QueryDirection::Forward,
            None,
        );

        let params = params
            .into_iter()
            .map(|key| key.to_raw_key())
            .collect::<Vec<_>>();
        let params: Vec<&dyn ToSql> = params.iter().map(|k| k as &dyn ToSql).collect();

        Ok(conn.query_row(
            &format!("EXPLAIN QUERY PLAN {sql}"),
            params.as_slice(),
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
    type Index<'coll> = SqliteCollection<'coll, Mode> where Self: 'coll;

    fn index(&self, name: &str) -> Result<Self::Index<'_>, Self::Error> {
        if !self.description.table_contains_index(&self.name, name) {
            return Err(Error::UnknownIndex {
                collection: self.name.to_string(),
                index: name.to_string(),
            });
        }

        Ok(SqliteCollection {
            is_index: true,
            name: self.name.clone(),
            key_column: name.to_string(),
            obj: self.obj.clone(),
            description: self.description.clone(),
            phantom: Default::default(),
        })
    }
}

#[async_trait]
impl<'tx, Mode: Send + Sync> ReadableCollection<'tx> for SqliteCollection<'tx, Mode>
where
    Mode: ReadMode + Sync,
{
    async fn get<K: KeyType + ?Sized, V: DeserializeOwned>(
        &self,
        key: &K,
    ) -> Result<Option<V>, Self::Error> {
        let conn = self.obj.lock()?;
        let sql = format!(
            "SELECT `data` FROM '{}' WHERE {} = ?;",
            self.name,
            self.qualified_key_column()
        );
        let mut statement = conn.prepare(&sql)?;
        let data = statement
            .query_row(params![key.to_raw_key()], |row| row.get::<_, String>(0))
            .optional()?;
        let result = data.map(|data| serde_json::from_str(&data)).transpose()?;

        Ok(result)
    }

    async fn contains_key<K: KeyType + ?Sized>(&self, key: &K) -> Result<bool, Self::Error> {
        let conn = self.obj.lock()?;
        let mut statement = conn.prepare(&format!(
            "SELECT EXISTS(SELECT 1 FROM '{}' WHERE {} = ?);",
            self.name,
            self.qualified_key_column()
        ))?;
        Ok(statement.query_row(params![key.to_raw_key()], |row| row.get(0))?)
    }

    async fn all_keys(&self) -> Result<Vec<String>, Self::Error> {
        let conn = self.obj.lock()?;
        let mut statement = conn.prepare(&format!(
            "SELECT {} FROM '{}'",
            self.qualified_key_column(),
            self.name
        ))?;
        let result = statement
            .query_map(params![], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    async fn get_all<Value: DeserializeOwned + Send>(
        &self,
        query: Query<impl KeyType>,
        direction: QueryDirection,
        limit: Option<usize>,
    ) -> Result<Vec<(String, Value)>, Self::Error> {
        self._get_all_filtered(
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
        query: Query<impl KeyType>,
        direction: QueryDirection,
        limit: Option<usize>,
        filter: impl FnMut(String, Value) -> Option<T> + SendUnlessWasm,
    ) -> Result<Vec<T>, Self::Error> {
        self._get_all_filtered(query, filter, direction, limit, false)
            .await
    }
}

impl<'tx, Mode> SqliteCollection<'tx, Mode>
where
    Mode: ReadMode + Sync,
{
    async fn _get_all_filtered<Key: KeyType, Value: DeserializeOwned + Send, T: Send>(
        &self,
        query: Query<Key>,
        mut filter: impl FnMut(String, Value) -> Option<T> + Send,
        direction: QueryDirection,
        limit: Option<usize>,
        limit_query: bool,
    ) -> Result<Vec<T>, Error> {
        let conn = self.obj.lock()?;
        let (sql, params) = query.to_sql(
            &self.name,
            &self.qualified_key_column(),
            direction,
            if limit_query { limit } else { None },
        );
        let params = params
            .into_iter()
            .map(|key| key.to_raw_key())
            .collect::<Vec<_>>();
        let params: Vec<&dyn ToSql> = params.iter().map(|k| k as &dyn ToSql).collect();

        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(params.as_slice(), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let limit = limit.unwrap_or(usize::MAX);
        if limit == 0 {
            return Ok(vec![]);
        }

        let mut result = vec![];

        for row in rows {
            let (key, data) = row?;
            let value = serde_json::from_str(&data)?;

            if let Some(transformed_value) = (filter)(key, value) {
                result.push(transformed_value)
            }

            if result.len() == limit {
                break;
            }
        }

        Ok(result)
    }
}

#[async_trait]
impl<'tx> WritableCollection<'tx> for SqliteCollection<'tx, ReadWrite> {
    fn add_index(&self, idx: IndexSpec) -> Result<(), Self::Error> {
        let conn = self.obj.lock()?;
        let index_type = if idx.unique { "UNIQUE INDEX" } else { "INDEX" };
        let sql = &format!(
            "CREATE {index_type} 'prose_{index_name}_idx' ON '{table_name}'(json_extract(`data`, '$.{index_column}'))",
            index_name = idx.key,
            table_name = self.name,
            index_column = idx.key
        );
        let mut statement = conn.prepare(&sql)?;
        statement.execute(params![])?;
        self.description.add_index(&self.name, &idx.key);
        Ok(())
    }

    async fn set<K: KeyType + ?Sized, V: Serialize + ?Sized + Send + Sync>(
        &self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error> {
        let conn = self.obj.lock()?;
        let mut statement = conn.prepare(&format!(
            r#"INSERT INTO "{}" (`key`, `data`) VALUES (?, ?)"#,
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
            r#"INSERT OR REPLACE INTO "{}" (`key`, `data`) VALUES (?, ?)"#,
            self.name
        ))?;
        statement.execute(params![key.to_raw_key(), &serde_json::to_string(value)?])?;
        Ok(())
    }

    fn delete<K: KeyType + ?Sized>(&self, key: &K) -> Result<(), Self::Error> {
        let conn = self.obj.lock()?;
        let mut statement =
            conn.prepare(&format!(r#"DELETE FROM {} WHERE `key` = ?"#, self.name))?;
        statement.execute(params![key.to_raw_key()])?;
        Ok(())
    }

    fn truncate(&self) -> Result<(), Self::Error> {
        let conn = self.obj.lock()?;
        let mut statement = conn.prepare(&format!(r#"DELETE FROM {}"#, self.name))?;
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
    fn database_description(&self) -> rusqlite::Result<DatabaseDescription>;
}

impl ConnectionExt for rusqlite::Connection {
    fn database_description(&self) -> rusqlite::Result<DatabaseDescription> {
        let mut tables_to_indexes_map = HashMap::new();

        // Order the rows so that type=table comes before type=index…
        let mut statement = self
            .prepare("SELECT `type`, `name`, `tbl_name` FROM sqlite_schema ORDER BY `type` DESC")?;
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

                    // e.g: prose_field_idx
                    let idx_name = &name[6..name.len() - 4];

                    let table_name = row.get::<_, String>(2)?;
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

impl<T: KeyType> Query<T> {
    fn to_sql(
        &self,
        table: &str,
        column: &str,
        direction: QueryDirection,
        limit: Option<usize>,
    ) -> (String, Vec<&T>) {
        let order = match direction {
            QueryDirection::Forward => "ASC",
            QueryDirection::Backward => "DESC",
        };

        let (predicate, params) = match self {
            Query::All => ("".to_string(), vec![]),

            Query::Range {
                start: Bound::Included(start),
                end: Bound::Included(end),
            } => (format!("{column} >= ? AND {column} <= ?"), vec![start, end]),
            Query::Range {
                start: Bound::Included(start),
                end: Bound::Excluded(end),
            } => (format!("{column} >= ? AND {column} < ?"), vec![start, end]),
            Query::Range {
                start: Bound::Included(start),
                end: Bound::Unbounded,
            } => (format!("{column} >= ?"), vec![start]),

            Query::Range {
                start: Bound::Excluded(start),
                end: Bound::Included(end),
            } => (format!("{column} > ? AND {column} <= ?"), vec![start, end]),
            Query::Range {
                start: Bound::Excluded(start),
                end: Bound::Excluded(end),
            } => (format!("{column} > ? AND {column} < ?"), vec![start, end]),
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

        let mut sql = format!("SELECT `key`, `data` FROM '{table}'");

        if !predicate.is_empty() {
            sql.push_str(&format!(" WHERE {predicate}"));
        }

        sql.push_str(&format!(" ORDER BY {column} {order}"));

        if let Some(limit) = limit {
            sql.push_str(&format!(" LIMIT {limit}"))
        }

        (sql, params)
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
