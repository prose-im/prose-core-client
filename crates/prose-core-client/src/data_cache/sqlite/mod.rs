use std::str::FromStr;

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};

pub use cache::{Connection, SQLiteCache};

mod cache;
mod contacts_cache;
mod message_cache;

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
