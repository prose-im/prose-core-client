use either::Either;
use jid::BareJid;
use rusqlite::types::FromSqlError;
use rusqlite::{params, params_from_iter};

use prose_core_lib::stanza::message;

use crate::cache::sqlite_data_cache::{repeat_vars, FromStrSql};
use crate::cache::MessageCache;
use crate::types::{MessageLike, Page};
use crate::SQLiteCache;

impl MessageCache for SQLiteCache {
    fn insert_messages<'a>(
        &self,
        messages: impl IntoIterator<Item = &'a MessageLike>,
    ) -> anyhow::Result<()> {
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

    fn load_messages_targeting<'a>(
        &self,
        conversation: &BareJid,
        targets: &[message::Id],
        newer_than: impl Into<Option<&'a message::Id>>,
        include_targeted_messages: bool,
    ) -> anyhow::Result<Vec<MessageLike>> {
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

    fn load_messages_before(
        &self,
        conversation: &BareJid,
        older_than: Option<&message::Id>,
        max_count: u32,
    ) -> anyhow::Result<Option<Page<MessageLike>>> {
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

    fn load_messages_after(
        &self,
        conversation: &BareJid,
        newer_than: &message::Id,
        max_count: Option<u32>,
    ) -> anyhow::Result<Vec<MessageLike>> {
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

    fn load_stanza_id(
        &self,
        conversation: &BareJid,
        message_id: &message::Id,
    ) -> anyhow::Result<Option<message::StanzaId>> {
        let conn = &*self.conn.lock().unwrap();
        let stanza_id = conn.query_row(
            "SELECT `stanza_id` FROM messages WHERE `id` = ?1 AND (`to` = ?2 OR `from` = ?2)",
            params![conversation.to_string(), message_id.as_ref()],
            |row| {
                Ok(row
                    .get::<_, Option<FromStrSql<message::StanzaId>>>(1)?
                    .map(|val| val.0))
            },
        )?;
        Ok(stanza_id)
    }
}

impl TryFrom<&rusqlite::Row<'_>> for MessageLike {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(MessageLike {
            id: row.get::<_, FromStrSql<message::Id>>(0)?.0,
            stanza_id: row
                .get::<_, Option<FromStrSql<message::StanzaId>>>(1)?
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::{TimeZone, Utc};
    use rusqlite::Connection;

    use crate::types::message_like::Payload;

    use super::*;

    #[test]
    fn test_load_messages_targeting() -> anyhow::Result<()> {
        let cache = SQLiteCache::open_with_connection(Connection::open_in_memory()?)?;

        let messages = [
            MessageLike {
                id: "1000".into(),
                stanza_id: None,
                target: None,
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 07, 16, 00, 00).unwrap(),
                payload: Payload::Message {
                    body: String::from(""),
                },
                is_first_message: false,
            },
            MessageLike {
                id: "1001".into(),
                stanza_id: None,
                target: None,
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 07, 17, 00, 00).unwrap(),
                payload: Payload::Message {
                    body: String::from(""),
                },
                is_first_message: false,
            },
            MessageLike {
                id: "1".into(),
                stanza_id: None,
                target: Some("1000".into()),
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 17, 00, 00).unwrap(),
                payload: Payload::Retraction,
                is_first_message: false,
            },
            MessageLike {
                id: "2".into(),
                stanza_id: None,
                target: Some("1001".into()),
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 17, 00, 00).unwrap(),
                payload: Payload::Retraction,
                is_first_message: false,
            },
            MessageLike {
                id: "3".into(),
                stanza_id: None,
                target: Some("2000".into()),
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 18, 00, 00).unwrap(),
                payload: Payload::Retraction,
                is_first_message: false,
            },
            MessageLike {
                id: "4".into(),
                stanza_id: None,
                target: Some("1000".into()),
                to: BareJid::from_str("b@prose.org").unwrap(),
                from: BareJid::from_str("a@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 19, 00, 00).unwrap(),
                payload: Payload::Retraction,
                is_first_message: false,
            },
            MessageLike {
                id: "5".into(),
                stanza_id: None,
                target: Some("1000".into()),
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("c@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 20, 00, 00).unwrap(),
                payload: Payload::Retraction,
                is_first_message: false,
            },
            MessageLike {
                id: "6".into(),
                stanza_id: None,
                target: Some("1000".into()),
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 21, 00, 00).unwrap(),
                payload: Payload::Retraction,
                is_first_message: false,
            },
        ];

        cache.insert_messages(&messages)?;

        assert_eq!(
            cache.load_messages_targeting(
                &BareJid::from_str("b@prose.org").unwrap(),
                &[message::Id::from("1000"), message::Id::from("1001")],
                &message::Id::from("1"),
                false
            )?,
            vec![
                messages[3].clone(),
                messages[5].clone(),
                messages[7].clone(),
            ]
        );

        assert_eq!(
            cache.load_messages_targeting(
                &BareJid::from_str("b@prose.org").unwrap(),
                &[message::Id::from("1000"), message::Id::from("1001")],
                None,
                true
            )?,
            vec![
                messages[0].clone(),
                messages[1].clone(),
                messages[2].clone(),
                messages[3].clone(),
                messages[5].clone(),
                messages[7].clone(),
            ]
        );

        Ok(())
    }

    #[test]
    fn test_load_messages_before() -> anyhow::Result<()> {
        let cache = SQLiteCache::open_with_connection(Connection::open_in_memory()?)?;

        let messages = [
            MessageLike {
                id: "1000".into(),
                stanza_id: Some("1".into()),
                target: None,
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 17, 00, 00).unwrap(),
                payload: Payload::Message {
                    body: String::from("msg1"),
                },
                is_first_message: true,
            },
            MessageLike {
                id: "2000".into(),
                stanza_id: Some("2".into()),
                target: None,
                to: BareJid::from_str("b@prose.org").unwrap(),
                from: BareJid::from_str("a@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 18, 00, 00).unwrap(),
                payload: Payload::Message {
                    body: String::from("msg2"),
                },
                is_first_message: false,
            },
            MessageLike {
                id: "3000".into(),
                stanza_id: Some("3".into()),
                target: None,
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 18, 00, 00).unwrap(),
                payload: Payload::Message {
                    body: String::from("msg3"),
                },
                is_first_message: false,
            },
            MessageLike {
                id: "4000".into(),
                stanza_id: Some("4".into()),
                target: None,
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 19, 00, 00).unwrap(),
                payload: Payload::Message {
                    body: String::from("msg4"),
                },
                is_first_message: false,
            },
            MessageLike {
                id: "5000".into(),
                stanza_id: Some("5".into()),
                target: None,
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("c@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 17, 00, 00).unwrap(),
                payload: Payload::Message {
                    body: String::from("msg5"),
                },
                is_first_message: false,
            },
        ];

        cache.insert_messages(&messages)?;

        assert_eq!(
            cache.load_messages_before(
                &BareJid::from_str("b@prose.org").unwrap(),
                Some(&message::Id::from("3000")),
                100,
            )?,
            Some(Page {
                is_complete: true,
                items: vec![messages[0].clone(), messages[1].clone()]
            })
        );

        assert_eq!(
            cache.load_messages_before(
                &BareJid::from_str("b@prose.org").unwrap(),
                Some(&message::Id::from("4000")),
                2,
            )?,
            Some(Page {
                is_complete: false,
                items: vec![messages[1].clone(), messages[2].clone()]
            })
        );

        assert_eq!(
            cache.load_messages_before(
                &BareJid::from_str("b@prose.org").unwrap(),
                Some(&message::Id::from("1000")),
                100,
            )?,
            Some(Page {
                is_complete: true,
                items: vec![]
            })
        );

        assert_eq!(
            cache.load_messages_before(
                &BareJid::from_str("c@prose.org").unwrap(),
                Some(&message::Id::from("5000")),
                100,
            )?,
            None
        );

        assert_eq!(
            cache.load_messages_before(&BareJid::from_str("b@prose.org").unwrap(), None, 2,)?,
            Some(Page {
                is_complete: false,
                items: vec![messages[2].clone(), messages[3].clone()]
            })
        );

        assert_eq!(
            cache.load_messages_before(&BareJid::from_str("d@prose.org").unwrap(), None, 100,)?,
            None
        );

        Ok(())
    }

    #[test]
    fn test_load_messages_after() -> anyhow::Result<()> {
        let cache = SQLiteCache::open_with_connection(Connection::open_in_memory()?)?;

        let messages = [
            MessageLike {
                id: "1000".into(),
                stanza_id: Some("1".into()),
                target: None,
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 17, 00, 00).unwrap(),
                payload: Payload::Message {
                    body: String::from("msg1"),
                },
                is_first_message: true,
            },
            MessageLike {
                id: "2000".into(),
                stanza_id: Some("2".into()),
                target: None,
                to: BareJid::from_str("b@prose.org").unwrap(),
                from: BareJid::from_str("a@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 18, 00, 00).unwrap(),
                payload: Payload::Message {
                    body: String::from("msg2"),
                },
                is_first_message: false,
            },
            MessageLike {
                id: "3000".into(),
                stanza_id: Some("3".into()),
                target: None,
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("c@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 18, 00, 00).unwrap(),
                payload: Payload::Message {
                    body: String::from("msg3"),
                },
                is_first_message: false,
            },
            MessageLike {
                id: "4000".into(),
                stanza_id: Some("4".into()),
                target: None,
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 08, 18, 00, 00).unwrap(),
                payload: Payload::Message {
                    body: String::from("msg4"),
                },
                is_first_message: false,
            },
        ];

        cache.insert_messages(&messages)?;

        assert_eq!(
            cache.load_messages_after(
                &BareJid::from_str("b@prose.org").unwrap(),
                &"4000".into(),
                None,
            )?,
            vec![messages[1].clone()]
        );

        assert_eq!(
            cache.load_messages_after(
                &BareJid::from_str("b@prose.org").unwrap(),
                &"1000".into(),
                None,
            )?,
            vec![messages[1].clone(), messages[3].clone()]
        );

        assert_eq!(
            cache.load_messages_after(
                &BareJid::from_str("b@prose.org").unwrap(),
                &"1000".into(),
                Some(1),
            )?,
            vec![messages[3].clone()]
        );

        Ok(())
    }
}
