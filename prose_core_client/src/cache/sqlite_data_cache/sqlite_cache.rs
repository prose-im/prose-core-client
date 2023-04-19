// TODO: Look into SQLite thread safety
// https://github.com/rusqlite/rusqlite/issues/393#user-content-fn-threads-d6886dc9aa33e26f0bb48e6eddf5854d
// https://sqlite.org/threadsafe.html

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};
use tracing::{debug, info};

use crate::cache::data_cache::DataCache;

pub struct SQLiteCache {
    pub(super) conn: Mutex<Connection>,
}

impl SQLiteCache {
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        Self::open_with_connection(Connection::open(path.as_ref().join("db.sqlite3"))?)
    }

    pub fn open_with_connection(conn: Connection) -> anyhow::Result<Self> {
        let mut conn = conn;
        conn.trace(Some(|query| {
            debug!("{}", query);
        }));
        Self::run_migrations(&mut conn)?;
        Self::create_temporary_presence_table(&conn)?;
        Self::create_temporary_chat_state_table(&conn)?;

        Ok(SQLiteCache {
            conn: Mutex::new(conn),
        })
    }
}

impl DataCache for SQLiteCache {
    fn delete_all(&self) -> anyhow::Result<()> {
        let conn = &*self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
        DELETE FROM roster_item;
        DELETE FROM user_profile;
        DELETE FROM avatar_metadata;
        DELETE FROM messages;
        DELETE FROM kv WHERE key != "version";
        "#,
        )?;
        Ok(())
    }
}

const DATABASE_VERSION: u8 = 2;

impl SQLiteCache {
    fn create_temporary_presence_table(conn: &Connection) -> anyhow::Result<()> {
        conn.execute(
            r#"
            CREATE TEMPORARY TABLE "presence" (
                "jid" TEXT PRIMARY KEY NOT NULL,
                "type" TEXT, 
                "show" TEXT, 
                "status" TEXT
            );"#,
            [],
        )?;
        Ok(())
    }

    fn create_temporary_chat_state_table(conn: &Connection) -> anyhow::Result<()> {
        conn.execute(
            r#"
            CREATE TEMPORARY TABLE "chat_states" (
                "jid" TEXT PRIMARY KEY NOT NULL,
                "state" TEXT NOT NULL, 
                "updated_at" DATETIME NOT NULL
            );"#,
            [],
        )?;
        Ok(())
    }

    fn run_migrations(conn: &mut Connection) -> anyhow::Result<()> {
        let version = Self::get_current_db_version(conn)?;

        info!(
            "Migrating database from version {:?} to {:?}…",
            version, DATABASE_VERSION
        );

        if version < 1 {
            conn.pragma_update(None, "journal_mode", "wal")?;
            Self::run_migration(conn, include_str!("../../../migrations/001_init.sql"), 1)?;
        }
        if version < 2 {
            conn.pragma_update(None, "foreign_keys", "ON")?;
            Self::run_migration(
                conn,
                include_str!("../../../migrations/002_add_messages.sql"),
                2,
            )?;
        }

        Ok(())
    }

    fn get_current_db_version(conn: &Connection) -> anyhow::Result<u8> {
        let kv_table_exists = conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'kv'",
            (),
            |row| row.get::<_, u32>(0),
        )? > 0;

        if !kv_table_exists {
            return Ok(0);
        }

        let version = conn
            .query_row(
                "SELECT `value` FROM 'kv' WHERE `key` = 'version'",
                (),
                |row| row.get::<_, u8>(0),
            )
            .optional()?;

        Ok(version.unwrap_or(0))
    }

    fn run_migration(conn: &mut Connection, sql: &str, version: u8) -> anyhow::Result<()> {
        let trx = conn.transaction()?;
        trx.execute_batch(sql)?;
        trx.execute(
            "INSERT INTO kv VALUES (?1, ?2) ON CONFLICT (key) DO UPDATE SET value = ?2",
            params!["version", version],
        )?;
        trx.commit()?;
        Ok(())
    }
}
