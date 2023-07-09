use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use jid::BareJid;
use microtype::Microtype;
use rusqlite::{params, OptionalExtension};
use xmpp_parsers::presence;

use prose_core_domain::Contact;
use prose_xmpp::stanza::avatar;
use prose_xmpp::stanza::message::ChatState;

use crate::cache::sqlite_data_cache::FromStrSql;
use crate::cache::ContactsCache;
use crate::domain_ext::Availability;
use crate::types::{roster, Address, AvatarMetadata, UserProfile};
use crate::SQLiteCache;

impl ContactsCache for SQLiteCache {
    fn has_valid_roster_items(&self) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let last_update = conn
            .query_row(
                "SELECT `value` FROM 'kv' WHERE `key` = 'roster_updated_at'",
                (),
                |row| row.get::<_, DateTime<Utc>>(0),
            )
            .optional()?;

        let Some(last_update) = last_update else {
            return Ok(false);
        };

        Ok(Utc::now() - last_update <= Duration::days(10))
    }

    fn insert_roster_items(&self, items: &[roster::Item]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let trx = (*conn).transaction()?;
        {
            let mut stmt = trx.prepare(
                r#"
            INSERT OR REPLACE INTO roster_item 
                (jid, subscription, groups) 
                VALUES (?1, ?2, ?3)
            "#,
            )?;
            for item in items {
                stmt.execute((
                    &item.jid.to_string(),
                    &item.subscription.to_string(),
                    &item.groups.join(","),
                ))?;
            }

            trx.execute(
                "INSERT OR REPLACE INTO kv VALUES (?1, ?2)",
                params!["roster_updated_at", Utc::now()],
            )?;
        }
        trx.commit()?;
        Ok(())
    }

    fn insert_user_profile(&self, jid: &BareJid, profile: &UserProfile) -> Result<()> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            INSERT OR REPLACE INTO user_profile 
                (jid, full_name, nickname, org, title, email, tel, url, locality, country, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )?;
        stmt.execute(params![
            &jid.to_string(),
            &profile.full_name,
            &profile.nickname,
            &profile.org,
            &profile.title,
            &profile.email,
            &profile.tel,
            &profile.url,
            profile.address.as_ref().map(|a| &a.locality),
            profile.address.as_ref().map(|a| &a.country),
            Utc::now()
        ])?;
        Ok(())
    }

    fn load_user_profile(&self, jid: &BareJid) -> Result<Option<UserProfile>> {
        let conn = &*self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT full_name, nickname, org, title, email, tel, url, locality, country 
                FROM user_profile 
                WHERE jid = ? AND updated_at >= ?
           "#,
        )?;

        let cache_max_age = Utc::now() - Duration::days(10);

        let profile = stmt
            .query_row(params![jid.to_string(), cache_max_age], |row| {
                let locality: Option<String> = row.get(7)?;
                let country: Option<String> = row.get(8)?;
                let mut address: Option<Address> = None;

                if locality.is_some() || country.is_some() {
                    address = Some(Address { locality, country })
                }

                Ok(UserProfile {
                    full_name: row.get(0)?,
                    nickname: row.get(1)?,
                    org: row.get(2)?,
                    title: row.get(3)?,
                    email: row.get(4)?,
                    tel: row.get(5)?,
                    url: row.get(6)?,
                    address,
                })
            })
            .optional()?;

        Ok(profile)
    }

    fn delete_user_profile(&self, jid: &BareJid) -> Result<()> {
        let conn = &*self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM user_profile WHERE jid = ?",
            params![jid.to_string()],
        )?;
        Ok(())
    }

    fn insert_avatar_metadata(&self, jid: &BareJid, metadata: &AvatarMetadata) -> Result<()> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO avatar_metadata \
                (jid, mime_type, checksum, width, height, updated_at) \
                VALUES (?, ?, ?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            &jid.to_string(),
            &metadata.mime_type,
            metadata.checksum.as_ref(),
            &metadata.width,
            &metadata.height,
            Utc::now(),
        ])?;
        Ok(())
    }

    fn load_avatar_metadata(&self, jid: &BareJid) -> Result<Option<AvatarMetadata>> {
        let conn = &*self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT mime_type, checksum, width, height, updated_at 
                FROM avatar_metadata
                WHERE jid = ? AND updated_at >= ?
           "#,
        )?;

        let cache_max_age = Utc::now() - Duration::minutes(60);

        let metadata = stmt
            .query_row(params![jid.to_string(), cache_max_age], |row| {
                Ok(AvatarMetadata {
                    mime_type: row.get(0)?,
                    checksum: row.get::<_, String>(1)?.into(),
                    width: row.get(2)?,
                    height: row.get(3)?,
                })
            })
            .optional()?;

        Ok(metadata)
    }

    fn insert_presence(
        &self,
        jid: &BareJid,
        kind: Option<presence::Type>,
        show: Option<presence::Show>,
        status: Option<String>,
    ) -> Result<()> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO presence \
                (jid, type, show, status) \
                VALUES (?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            &jid.to_string(),
            kind.as_ref().map(|kind| kind.to_string()),
            show.as_ref().map(|show| show.to_string()),
            status
        ])?;
        Ok(())
    }

    fn insert_chat_state(&self, jid: &BareJid, chat_state: &ChatState) -> Result<()> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO chat_states (jid, state, updated_at) VALUES (?, ?, ?)",
        )?;
        stmt.execute(params![
            &jid.to_string(),
            &chat_state.to_string(),
            Utc::now()
        ])?;
        Ok(())
    }
    fn load_chat_state(&self, jid: &BareJid) -> Result<Option<ChatState>> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT state, updated_at FROM chat_states WHERE jid = ?")?;
        let row = stmt
            .query_row([&jid.to_string()], |row| {
                Ok((
                    row.get::<_, FromStrSql<ChatState>>(0)?.0,
                    row.get::<_, DateTime<Utc>>(1)?,
                ))
            })
            .optional()?;

        let Some(row) = row else { return Ok(None) };

        // If the chat state was composing but is older than 30 seconds we consider the actual state
        // to be 'active' (i.e. not currently typing).
        if row.0 == ChatState::Composing && Utc::now() - row.1 > Duration::seconds(30) {
            return Ok(Some(ChatState::Active));
        }

        Ok(Some(row.0))
    }

    fn load_contacts(&self) -> Result<Vec<(Contact, Option<avatar::ImageId>)>> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT
                roster_item.jid,
                roster_item.groups, 
                user_profile.full_name, 
                user_profile.nickname, 
                avatar_metadata.checksum, 
                COUNT(presence.jid) AS presence_count,
                presence.type, 
                presence.show, 
                presence.status
            FROM roster_item
            LEFT JOIN user_profile ON roster_item.jid = user_profile.jid
            LEFT JOIN avatar_metadata ON roster_item.jid = avatar_metadata.jid
            LEFT JOIN presence ON roster_item.jid = presence.jid
            GROUP BY roster_item.jid;
            "#,
        )?;

        let contacts = stmt
            .query_map([], |row| {
                let jid = row.get::<_, FromStrSql<BareJid>>(0)?.0;
                let groups: Vec<String> = row
                    .get::<_, String>(1)?
                    .split(",")
                    .map(Into::into)
                    .collect();
                let full_name: Option<String> = row.get(2)?;
                let nickname: Option<String> = row.get(3)?;
                let checksum: Option<avatar::ImageId> =
                    row.get::<_, Option<String>>(4)?.map(Into::into);
                let presence_count: u32 = row.get(5)?;
                let presence_kind: Option<presence::Type> =
                    row.get::<_, Option<FromStrSql<_>>>(6)?.map(|o| o.0);
                let presence_show: Option<presence::Show> =
                    row.get::<_, Option<FromStrSql<_>>>(7)?.map(|o| o.0);
                let status: Option<String> = row.get(8)?;

                let availability = if presence_count > 0 {
                    Availability::from((presence_kind, presence_show)).into_inner()
                } else {
                    prose_core_domain::Availability::Unavailable
                };

                Ok((
                    Contact {
                        jid: jid.clone(),
                        name: full_name.or(nickname).unwrap_or(jid.to_string()),
                        avatar: None,
                        availability,
                        status,
                        groups,
                    },
                    checksum,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(contacts)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rusqlite::Connection;
    use xmpp_parsers::presence::Show;

    use prose_core_domain::Availability;

    use crate::types::roster::Subscription;

    use super::*;

    #[test]
    fn test_presence() -> Result<()> {
        let cache = SQLiteCache::open_with_connection(Connection::open_in_memory()?)?;
        let jid_a = BareJid::from_str("a@prose.org").unwrap();
        let jid_b = BareJid::from_str("b@prose.org").unwrap();

        cache.insert_roster_items(&[
            roster::Item {
                jid: jid_a.clone(),
                subscription: Subscription::Both,
                groups: vec![],
            },
            roster::Item {
                jid: jid_b.clone(),
                subscription: Subscription::Both,
                groups: vec![],
            },
        ])?;

        // If we didn't receive a presence yet the contact should be considered unavailable.
        // If we did however receive an empty presence the contact should be considered
        // available, because of https://datatracker.ietf.org/doc/html/rfc6121#section-4.7.1
        cache.insert_presence(&jid_b, None, None, None)?;

        assert_eq!(
            cache
                .load_contacts()?
                .into_iter()
                .map(|c| c.0)
                .collect::<Vec<_>>(),
            vec![
                Contact {
                    jid: jid_a.clone(),
                    name: jid_a.to_string(),
                    avatar: None,
                    availability: Availability::Unavailable,
                    status: None,
                    groups: vec![String::from("")],
                },
                Contact {
                    jid: jid_b.clone(),
                    name: jid_b.to_string(),
                    avatar: None,
                    availability: Availability::Available,
                    status: None,
                    groups: vec![String::from("")],
                }
            ]
        );

        // And for good measure insert some non-empty values
        cache.insert_presence(&jid_a, None, Some(Show::Dnd), Some(String::from("AFK!")))?;
        assert_eq!(
            cache
                .load_contacts()?
                .into_iter()
                .map(|c| c.0)
                .collect::<Vec<_>>(),
            vec![
                Contact {
                    jid: jid_a.clone(),
                    name: jid_a.to_string(),
                    avatar: None,
                    availability: Availability::DoNotDisturb,
                    status: Some(String::from("AFK!")),
                    groups: vec![String::from("")],
                },
                Contact {
                    jid: jid_b.clone(),
                    name: jid_b.to_string(),
                    avatar: None,
                    availability: Availability::Available,
                    status: None,
                    groups: vec![String::from("")],
                }
            ]
        );

        Ok(())
    }
}

trait Stringify {
    fn to_string(&self) -> String;
}

impl Stringify for presence::Type {
    fn to_string(&self) -> String {
        use presence::Type;

        match self {
            Type::None => "",
            Type::Error => "error",
            Type::Probe => "probe",
            Type::Subscribe => "subscribe",
            Type::Subscribed => "subscribed",
            Type::Unavailable => "unavailable",
            Type::Unsubscribe => "unsubscribe",
            Type::Unsubscribed => "unsubscribed",
        }
        .to_string()
    }
}

impl Stringify for presence::Show {
    fn to_string(&self) -> String {
        use presence::Show;

        match self {
            Show::Away => "away",
            Show::Chat => "chat",
            Show::Dnd => "dnd",
            Show::Xa => "xa",
        }
        .to_string()
    }
}
