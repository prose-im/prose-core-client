// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use either::Either;
use jid::BareJid;
use rusqlite::types::FromSqlError;
use rusqlite::{params, params_from_iter, OptionalExtension};

use prose_xmpp::stanza::message;
use prose_xmpp::stanza::message::stanza_id;

use crate::data_cache::sqlite::cache::SQLiteCacheError;
use crate::data_cache::sqlite::{repeat_vars, FromStrSql, SQLiteCache};
use crate::data_cache::MessageCache;
use crate::types::{MessageLike, Page};

type Result<T, E = SQLiteCacheError> = std::result::Result<T, E>;

#[async_trait]
impl MessageCache for SQLiteCache {
    type Error = SQLiteCacheError;

    async fn insert_messages<'a>(
        &self,
        messages: impl IntoIterator<Item = &'a MessageLike> + Send,
    ) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let trx = (*conn).transaction()?;
        {
            let mut stmt = trx.prepare(
        r#"
                INSERT OR REPLACE INTO messages
                    (`id`, `stanza_id`, `target`, `to`, `from`, `timestamp`, `payload`, `is_first_message`)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
      )?;
            for msg in messages {
                stmt.execute(params![
                    msg.id.to_string(),
                    msg.stanza_id.as_ref().map(|id| id.to_string()),
                    msg.target.as_ref().map(|t| t.to_string()),
                    msg.to.to_string(),
                    msg.from.to_string(),
                    msg.timestamp,
                    serde_json::to_string(&msg.payload)?,
                    msg.is_first_message
                ])?;
            }
        }
        trx.commit()?;
        Ok(())
    }

    async fn load_messages_targeting<'a>(
        &self,
        conversation: &BareJid,
        targets: &[message::Id],
        newer_than: impl Into<Option<&'a message::Id>> + Send,
        include_targeted_messages: bool,
    ) -> Result<Vec<MessageLike>> {
        if targets.is_empty() {
            return Ok(vec![]);
        }

        let conn = &*self.conn.lock().unwrap();
        let repeated_vars = repeat_vars(targets.len());

        let (mut where_clause, targets_iter) = if include_targeted_messages {
            (
                format!(
                    "(`target` IN ({}) OR `id` IN ({})) ",
                    repeated_vars, repeated_vars
                ),
                Either::Left(
                    targets
                        .iter()
                        .map(AsRef::as_ref)
                        .chain(targets.iter().map(AsRef::as_ref)),
                ),
            )
        } else {
            (
                format!("`target` IN ({}) ", repeated_vars),
                Either::Right(targets.iter().map(AsRef::as_ref)),
            )
        };

        let mut params = Vec::<&str>::with_capacity(4);

        if let Some(newer_than) = newer_than.into() {
            where_clause.push_str(
                "AND `id` != ? AND `timestamp` >= (SELECT timestamp FROM messages WHERE id = ?)",
            );
            params.extend_from_slice(&[newer_than.as_ref(), newer_than.as_ref()]);
        }

        let sql = format!(
            r#"
            SELECT
              `id`,
              `stanza_id`,
              `target`,
              `to`,
              `from`,
              `timestamp`,
              `payload`,
              `is_first_message`
            FROM messages
            WHERE {} AND (`to` = ? OR `from` = ?)
            ORDER BY `timestamp` ASC, `rowid` ASC
           "#,
            where_clause
        );

        let conversation = conversation.to_string();
        params.extend_from_slice(&[&conversation, &conversation]);

        let mut stmt = conn.prepare(&sql)?;
        let params_iter = targets_iter.chain(params.into_iter());

        let messages = stmt
            .query_map(rusqlite::params_from_iter(params_iter), |row| {
                MessageLike::try_from(row)
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    async fn load_messages_before(
        &self,
        conversation: &BareJid,
        older_than: Option<&message::Id>,
        max_count: u32,
    ) -> Result<Option<Page<MessageLike>>> {
        let conn = &*self.conn.lock().unwrap();

        let mut sql = String::from(
            r#"
            SELECT
              `id`,
              `stanza_id`,
              `target`,
              `to`,
              `from`,
              `timestamp`,
              `payload`,
              `is_first_message`
            FROM messages
            WHERE (`to` = ?1 OR `from` = ?1)
           "#,
        );

        let conversation = conversation.to_string();
        let max_count = max_count.to_string();
        let mut params: Vec<&str> = vec![&conversation, &max_count];

        if let Some(older_than) = older_than {
            sql.push_str(
                r#"
                AND `id` != ?3
                AND `timestamp` <= (SELECT timestamp FROM messages WHERE id = ?3)
             "#,
            );
            params.push(&older_than.as_ref());
        }

        sql.push_str(
            r#"
            ORDER BY `timestamp` DESC, `rowid` DESC
            LIMIT ?2
        "#,
        );

        let conversation = conversation.to_string();
        let mut stmt = conn.prepare(&sql)?;

        let mut messages = stmt
            .query_map(params_from_iter(params.into_iter()), |row| {
                MessageLike::try_from(row)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        // We're reversing the array since we sort in the "wrong" order so that our LIMIT works as
        // expected.
        messages.reverse();

        // We've found some matching messages…
        if !messages.is_empty() {
            return Ok(Some(Page {
                is_complete: messages.iter().any(|message| message.is_first_message),
                items: messages,
            }));
        }

        // Since we didn't find any messages we need to find out if we hadn't cached the requested
        // page or if request was beyond the first message.
        let mut stmt = conn.prepare(
            r#"
        SELECT EXISTS(
          SELECT 1
          FROM messages
          WHERE (`to` = ?1 OR `from` = ?1)
          AND is_first_message = 1
        );
      "#,
        )?;

        let first_message_is_known =
            stmt.query_row(params![&conversation], |row| row.get::<_, bool>(0))?;

        if first_message_is_known {
            Ok(Some(Page {
                is_complete: true,
                items: vec![],
            }))
        } else {
            Ok(None)
        }
    }

    async fn load_messages_after(
        &self,
        conversation: &BareJid,
        newer_than: &message::Id,
        max_count: Option<u32>,
    ) -> Result<Vec<MessageLike>> {
        let conn = &*self.conn.lock().unwrap();

        let mut sql = String::from(
            r#"
            SELECT
              `id`,
              `stanza_id`,
              `target`,
              `to`,
              `from`,
              `timestamp`,
              `payload`,
              `is_first_message`
            FROM messages
            WHERE (`to` = ?1 OR `from` = ?1)
              AND `id` != ?2
              AND `timestamp` >= (SELECT timestamp FROM messages WHERE id = ?2)
            ORDER BY `timestamp` DESC, `rowid` DESC
           "#,
        );

        let conversation = conversation.to_string();
        let max_count = max_count.map(|c| c.to_string());

        let mut params: Vec<&str> = vec![&conversation, newer_than.as_ref()];

        if let Some(max_count) = &max_count {
            sql.push_str("LIMIT ?3");
            params.push(max_count);
        }

        let mut stmt = conn.prepare(&sql)?;
        let mut messages = stmt
            .query_map(params_from_iter(params.into_iter()), |row| {
                MessageLike::try_from(row)
            })?
            .collect::<Result<Vec<_>, _>>()?;
        messages.reverse();
        Ok(messages)
    }

    async fn load_stanza_id(
        &self,
        conversation: &BareJid,
        message_id: &message::Id,
    ) -> Result<Option<stanza_id::Id>> {
        let conn = &*self.conn.lock().unwrap();
        let stanza_id = conn.query_row(
            "SELECT `stanza_id` FROM messages WHERE `id` = ?1 AND (`to` = ?2 OR `from` = ?2)",
            params![conversation.to_string(), message_id.as_ref()],
            |row| {
                Ok(row
                    .get::<_, Option<FromStrSql<stanza_id::Id>>>(1)?
                    .map(|val| val.0))
            },
        )?;
        Ok(stanza_id)
    }

    async fn save_draft(&self, conversation: &BareJid, text: Option<&str>) -> Result<()> {
        let conn = &*self.conn.lock().unwrap();

        if let Some(text) = text {
            let mut stmt =
                conn.prepare("INSERT OR REPLACE INTO `drafts` (`jid`, `text`) VALUES (?, ?)")?;
            stmt.execute(params![conversation.to_string(), text])?;
        } else {
            let mut stmt = conn.prepare("DELETE FROM `drafts` WHERE `jid` = ?")?;
            stmt.execute(params![conversation.to_string()])?;
        }

        Ok(())
    }

    async fn load_draft(&self, conversation: &BareJid) -> Result<Option<String>> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT `text` FROM `drafts` WHERE `jid` = ?")?;
        Ok(stmt
            .query_row(params![conversation.to_string()], |row| Ok(row.get(0)?))
            .optional()?)
    }
}

impl TryFrom<&rusqlite::Row<'_>> for MessageLike {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(MessageLike {
            id: row.get::<_, FromStrSql<message::Id>>(0)?.0,
            stanza_id: row
                .get::<_, Option<FromStrSql<stanza_id::Id>>>(1)?
                .map(|val| val.0),
            target: row
                .get::<_, Option<FromStrSql<message::Id>>>(2)?
                .map(|t| t.0),
            to: row.get::<_, FromStrSql<BareJid>>(3)?.0,
            from: row.get::<_, FromStrSql<BareJid>>(4)?.0,
            timestamp: row.get(5)?,
            payload: serde_json::from_str(&row.get::<_, String>(6)?)
                .map_err(|_| FromSqlError::InvalidType)?,
            is_first_message: row.get(7)?,
        })
    }
}
