// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ValueRef};

pub use cache::{Connection, SQLiteCache, SQLiteCacheError};

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
