use anyhow::Result;
use jid::BareJid;
use std::str::FromStr;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as async_test;

#[cfg(target_arch = "wasm32")]
use prose_core_client::data_cache::indexed_db::IndexedDBDataCache;
#[cfg(not(target_arch = "wasm32"))]
use prose_core_client::data_cache::sqlite::{Connection, SQLiteCache};
use prose_core_client::data_cache::{ContactsCache, DataCache};
use prose_core_client::types::roster::Subscription;
use prose_core_client::types::{roster, AccountSettings, Availability, Contact};
#[cfg(not(target_arch = "wasm32"))]
use tokio::test as async_test;
use xmpp_parsers::presence::Show;

#[cfg(not(target_arch = "wasm32"))]
async fn cache() -> Result<SQLiteCache> {
    Ok(SQLiteCache::open_with_connection(
        Connection::open_in_memory()?,
    )?)
}

#[cfg(target_arch = "wasm32")]
async fn cache() -> Result<IndexedDBDataCache> {
    Ok(IndexedDBDataCache::new().await?)
}

#[async_test]
async fn test_save_and_load_account_settings() -> Result<()> {
    let cache = cache().await?;

    assert_eq!(cache.load_account_settings().await?, None);

    let settings = AccountSettings {
        availability: Availability::Away,
    };

    cache.save_account_settings(&settings).await?;
    assert_eq!(cache.load_account_settings().await?, Some(settings));

    Ok(())
}

#[async_test]
async fn test_presence() -> Result<()> {
    let cache = cache().await?;
    let jid_a = BareJid::from_str("a@prose.org").unwrap();
    let jid_b = BareJid::from_str("b@prose.org").unwrap();

    cache
        .insert_roster_items(&[
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
        ])
        .await?;

    // If we didn't receive a presence yet the contact should be considered unavailable.
    // If we did however receive an empty presence the contact should be considered
    // available, because of https://datatracker.ietf.org/doc/html/rfc6121#section-4.7.1
    cache.insert_presence(&jid_b, None, None, None).await?;

    assert_eq!(
        cache
            .load_contacts()
            .await?
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
    cache
        .insert_presence(&jid_a, None, Some(Show::Dnd), Some(String::from("AFK!")))
        .await?;
    assert_eq!(
        cache
            .load_contacts()
            .await?
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
