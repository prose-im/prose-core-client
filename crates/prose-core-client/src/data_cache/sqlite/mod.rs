use std::str::FromStr;

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};

pub use sqlite_cache::{Connection, SQLiteCache};

mod sqlite_cache;
mod sqlite_contacts_cache;
mod sqlite_message_cache;

pub(self) fn repeat_vars(count: usize) -> String {
    let mut s = "?,".repeat(count);
    // Remove trailing comma
    s.pop();
    s
}

pub(self) struct FromStrSql<T>(pub T);

impl<T: FromStr> FromSql for FromStrSql<T> {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        T::from_str(value.as_str()?)
            .map_err(|_| FromSqlError::InvalidType)
            .map(FromStrSql)
    }
}
