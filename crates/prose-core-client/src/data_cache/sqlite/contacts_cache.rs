// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use jid::BareJid;
use rusqlite::{params, OptionalExtension};

use prose_xmpp::stanza::message::ChatState;

use crate::data_cache::sqlite::cache::SQLiteCacheError;
use crate::data_cache::sqlite::{FromStrSql, SQLiteCache};
use crate::data_cache::ContactsCache;
use crate::types::{
    presence, roster, Address, Availability, AvatarMetadata, Contact, Presence, UserActivity,
    UserProfile,
};

type Result<T, E = SQLiteCacheError> = std::result::Result<T, E>;

#[async_trait]
impl ContactsCache for SQLiteCache {
    type Error = SQLiteCacheError;

    async fn set_roster_update_time(
        &self,
        timestamp: &DateTime<Utc>,
    ) -> std::result::Result<(), Self::Error> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare("INSERT OR REPLACE INTO kv VALUES (?1, ?2)")?;
        stmt.execute(params!["roster_updated_at", timestamp])?;
        Ok(())
    }

    async fn roster_update_time(&self) -> std::result::Result<Option<DateTime<Utc>>, Self::Error> {
        let conn = &*self.conn.lock().unwrap();
        let last_update = conn
            .query_row(
                "SELECT `value` FROM 'kv' WHERE `key` = 'roster_updated_at'",
                (),
                |row| row.get::<_, DateTime<Utc>>(0),
            )
            .optional()?;
        Ok(last_update)
    }

    async fn insert_roster_items(&self, items: &[roster::Item]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let trx = (*conn).transaction()?;
        {
            let mut stmt = trx.prepare(
                r#"
            INSERT OR REPLACE INTO roster_item
                (`jid`, `name`, `subscription`, `group`)
                VALUES (?1, ?2, ?3, ?4)
            "#,
            )?;
            for item in items {
                stmt.execute((
                    &item.jid.to_string(),
                    &item.name,
                    &item.subscription.to_string(),
                    &item.group.to_string(),
                ))?;
            }
        }
        trx.commit()?;
        Ok(())
    }

    async fn insert_user_profile(&self, jid: &BareJid, profile: &UserProfile) -> Result<()> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            INSERT OR REPLACE INTO user_profile
                (jid, first_name, last_name, nickname, org, role, title, email, tel, url, locality, country, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )?;
        stmt.execute(params![
            &jid.to_string(),
            &profile.first_name,
            &profile.last_name,
            &profile.nickname,
            &profile.org,
            &profile.role,
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

    async fn load_user_profile(&self, jid: &BareJid) -> Result<Option<UserProfile>> {
        let conn = &*self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT first_name, last_name, nickname, org, role, title, email, tel, url, locality, country
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
                    first_name: row.get(0)?,
                    last_name: row.get(1)?,
                    nickname: row.get(2)?,
                    org: row.get(3)?,
                    role: row.get(4)?,
                    title: row.get(5)?,
                    email: row.get(6)?,
                    tel: row.get(7)?,
                    url: row.get(8)?,
                    address,
                })
            })
            .optional()?;

        Ok(profile)
    }

    async fn delete_user_profile(&self, jid: &BareJid) -> Result<()> {
        let conn = &*self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM user_profile WHERE jid = ?",
            params![jid.to_string()],
        )?;
        Ok(())
    }

    async fn insert_avatar_metadata(&self, jid: &BareJid, metadata: &AvatarMetadata) -> Result<()> {
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

    async fn load_avatar_metadata(&self, jid: &BareJid) -> Result<Option<AvatarMetadata>> {
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

    async fn insert_presence(&self, jid: &BareJid, presence: &Presence) -> Result<()> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO presence \
                (jid, type, show, status) \
                VALUES (?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            &jid.to_string(),
            presence.kind.as_ref().map(|kind| kind.to_string()),
            presence.show.as_ref().map(|show| show.to_string()),
            presence.status
        ])?;
        Ok(())
    }

    async fn insert_user_activity(
        &self,
        jid: &BareJid,
        user_activity: &Option<UserActivity>,
    ) -> Result<(), Self::Error> {
        let conn = &*self.conn.lock().unwrap();

        let Some(user_activity) = user_activity else {
            conn.execute(
                "DELETE FROM user_activity WHERE jid = ?",
                params![jid.to_string()],
            )?;
            return Ok(());
        };

        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO user_activity \
                (jid, emoji, status) \
                VALUES (?, ?, ?)",
        )?;
        stmt.execute(params![
            &jid.to_string(),
            user_activity.emoji,
            user_activity.status
        ])?;
        Ok(())
    }

    async fn insert_chat_state(&self, jid: &BareJid, chat_state: &ChatState) -> Result<()> {
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

    async fn load_chat_state(&self, jid: &BareJid) -> Result<Option<ChatState>> {
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

    async fn load_contacts(&self) -> Result<Vec<Contact>> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT
                roster_item.jid,
                roster_item.name,
                roster_item.subscription,
                roster_item.`group`,
                user_profile.first_name,
                user_profile.last_name,
                user_profile.nickname,
                COUNT(presence.jid) AS presence_count,
                presence.type,
                presence.show,
                user_activity.emoji,
                user_activity.status
            FROM roster_item
            LEFT JOIN user_profile ON roster_item.jid = user_profile.jid
            LEFT JOIN presence ON roster_item.jid = presence.jid
            LEFT JOIN user_activity ON roster_item.jid = user_activity.jid
            GROUP BY roster_item.jid;
            "#,
        )?;

        let contacts = stmt
            .query_map([], |row| {
                let roster_item = roster::Item {
                    jid: row.get::<_, FromStrSql<BareJid>>(0)?.0,
                    name: row.get(1)?,
                    subscription: row.get::<_, FromStrSql<roster::Subscription>>(2)?.0,
                    group: row.get::<_, FromStrSql<roster::Group>>(3)?.0,
                };

                let user_profile = Some(UserProfile {
                    first_name: row.get(4)?,
                    last_name: row.get(5)?,
                    nickname: row.get(6)?,
                    org: None,
                    role: None,
                    title: None,
                    email: None,
                    tel: None,
                    url: None,
                    address: None,
                });

                let presence_count: u32 = row.get(7)?;
                let presence_kind: Option<presence::Type> =
                    row.get::<_, Option<FromStrSql<_>>>(8)?.map(|o| o.0);
                let presence_show: Option<presence::Show> =
                    row.get::<_, Option<FromStrSql<_>>>(9)?.map(|o| o.0);

                let availability = (presence_count > 0).then(|| {
                    Availability::from((presence_kind.map(|v| v.0), presence_show.map(|v| v.0)))
                });

                let emoji: Option<String> = row.get(10)?;
                let status: Option<String> = row.get(11)?;

                let activity = emoji.map(|emoji| UserActivity { emoji, status });

                Ok(Contact::from((
                    roster_item,
                    user_profile,
                    availability,
                    activity,
                )))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(contacts)
    }
}
