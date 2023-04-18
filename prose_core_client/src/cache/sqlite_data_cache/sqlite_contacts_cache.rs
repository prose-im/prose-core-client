use chrono::{DateTime, Duration, Utc};
use jid::BareJid;
use microtype::Microtype;
use rusqlite::{params, OptionalExtension};

use prose_core_domain::Contact;
use prose_core_lib::modules::profile::avatar::ImageId;
use prose_core_lib::modules::roster;
use prose_core_lib::stanza::presence;

use crate::cache::sqlite_data_cache::FromStrSql;
use crate::cache::ContactsCache;
use crate::domain_ext::Availability;
use crate::types::{Address, AvatarMetadata, RosterItem, UserProfile};
use crate::SQLiteCache;

impl ContactsCache for SQLiteCache {
    fn has_valid_roster_items(&self) -> anyhow::Result<bool> {
        let conn = self.conn.lock().unwrap();

        let last_update = conn
            .query_row(
                "SELECT `value` FROM 'kv' WHERE `key` = 'roster_updated_at'",
                (),
                |row| row.get::<_, DateTime<Utc>>(0),
            )
            .optional()?;

        let Some(last_update) = last_update else {
            return Ok(false)
        };

        Ok(Utc::now() - last_update <= Duration::minutes(60))
    }

    fn insert_roster_items(&self, items: &[RosterItem]) -> anyhow::Result<()> {
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

    fn load_roster_items(&self) -> anyhow::Result<Option<Vec<RosterItem>>> {
        if !self.has_valid_roster_items()? {
            return Ok(None);
        }

        let conn = &*self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT jid, subscription, groups FROM roster_item")?;
        let items = stmt
            .query_map([], |row| {
                Ok(RosterItem {
                    jid: row.get::<_, FromStrSql<BareJid>>(0)?.0,
                    subscription: row.get::<_, FromStrSql<roster::Subscription>>(1)?.0,
                    groups: row
                        .get::<_, String>(2)?
                        .split(",")
                        .map(Into::into)
                        .collect(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(items))
    }

    fn insert_user_profile(&self, jid: &BareJid, profile: &UserProfile) -> anyhow::Result<()> {
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

    fn load_user_profile(&self, jid: &BareJid) -> anyhow::Result<Option<UserProfile>> {
        let conn = &*self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT full_name, nickname, org, title, email, tel, url, locality, country, updated_at 
                FROM user_profile 
                WHERE jid = ? AND updated_at >= ?
           "#,
        )?;

        let cache_max_age = Utc::now() - Duration::minutes(60);

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

    fn insert_avatar_metadata(
        &self,
        jid: &BareJid,
        metadata: &AvatarMetadata,
    ) -> anyhow::Result<()> {
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

    fn load_avatar_metadata(&self, jid: &BareJid) -> anyhow::Result<Option<AvatarMetadata>> {
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
        kind: Option<presence::Kind>,
        show: Option<presence::Show>,
        status: Option<String>,
    ) -> anyhow::Result<()> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO presence \
                (jid, type, show, status) \
                VALUES (?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            &jid.to_string(),
            kind.as_ref().map(ToString::to_string),
            show.as_ref().map(ToString::to_string),
            status
        ])?;
        Ok(())
    }

    fn load_contacts(&self) -> anyhow::Result<Vec<(Contact, Option<ImageId>)>> {
        let conn = &*self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT
                roster_item.jid,
                roster_item.groups, 
                user_profile.full_name, 
                user_profile.nickname, 
                avatar_metadata.checksum, 
                presence.type, 
                presence.show, 
                presence.status
            FROM roster_item
            LEFT JOIN user_profile ON roster_item.jid = user_profile.jid
            LEFT JOIN avatar_metadata ON roster_item.jid = avatar_metadata.jid
            LEFT JOIN presence ON roster_item.jid = presence.jid;
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
                let checksum: Option<ImageId> = row.get::<_, Option<String>>(4)?.map(Into::into);
                let presence_kind: Option<presence::Kind> =
                    row.get::<_, Option<FromStrSql<_>>>(5)?.map(|o| o.0);
                let presence_show: Option<presence::Show> =
                    row.get::<_, Option<FromStrSql<_>>>(6)?.map(|o| o.0);
                let status: Option<String> = row.get(7)?;

                Ok((
                    Contact {
                        jid: jid.clone(),
                        name: full_name.or(nickname).unwrap_or(jid.to_string()),
                        avatar: None,
                        availability: Availability::from((presence_kind, presence_show))
                            .into_inner(),
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
