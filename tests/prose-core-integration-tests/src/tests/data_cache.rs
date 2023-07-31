use std::str::FromStr;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use jid::BareJid;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test as async_test;
use xmpp_parsers::presence::Show;

#[cfg(target_arch = "wasm32")]
use prose_core_client::data_cache::indexed_db::IndexedDBDataCache;
#[cfg(not(target_arch = "wasm32"))]
use prose_core_client::data_cache::sqlite::{Connection, SQLiteCache};
use prose_core_client::data_cache::{AccountCache, ContactsCache, MessageCache};
use prose_core_client::types::message_like::Payload;
use prose_core_client::types::roster::Subscription;
use prose_core_client::types::{
    presence, roster, AccountSettings, Availability, Contact, MessageLike, Page, Presence,
    UserActivity,
};
use prose_xmpp::stanza::message;
#[cfg(not(target_arch = "wasm32"))]
use tokio::test as async_test;

#[cfg(not(target_arch = "wasm32"))]
async fn cache() -> Result<SQLiteCache> {
    Ok(SQLiteCache::open_with_connection(
        Connection::open_in_memory()?,
    )?)
}

#[cfg(target_arch = "wasm32")]
async fn cache() -> Result<IndexedDBDataCache> {
    let cache = IndexedDBDataCache::new().await?;
    cache.delete_all().await?;
    Ok(cache)
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
async fn test_set_roster_update_time() -> Result<()> {
    let cache = cache().await?;

    let date1 = Utc.with_ymd_and_hms(2023, 7, 20, 18, 00, 00).unwrap();
    let date2 = Utc.with_ymd_and_hms(2023, 7, 19, 17, 30, 10).unwrap();

    assert_eq!(cache.roster_update_time().await?, None);

    cache.set_roster_update_time(&date1).await?;
    assert_eq!(cache.roster_update_time().await?, Some(date1));

    cache.set_roster_update_time(&date2).await?;
    assert_eq!(cache.roster_update_time().await?, Some(date2));

    cache.delete_all().await?;
    assert_eq!(cache.roster_update_time().await?, None);

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
    cache
        .insert_presence(
            &jid_b,
            &Presence {
                kind: None,
                show: None,
                status: Some("Should be ignored".to_string()),
            },
        )
        .await?;

    cache
        .insert_user_activity(
            &jid_b,
            &Some(UserActivity {
                emoji: "🍰".to_string(),
                status: Some("Baking cake".to_string()),
            }),
        )
        .await?;

    assert_eq!(
        cache.load_contacts().await?.into_iter().collect::<Vec<_>>(),
        vec![
            Contact {
                jid: jid_a.clone(),
                name: jid_a.to_string(),
                availability: Availability::Unavailable,
                activity: None,
                groups: vec![String::from("")],
            },
            Contact {
                jid: jid_b.clone(),
                name: jid_b.to_string(),
                availability: Availability::Available,
                activity: Some(UserActivity {
                    emoji: "🍰".to_string(),
                    status: Some("Baking cake".to_string()),
                }),
                groups: vec![String::from("")],
            }
        ]
    );

    cache
        .insert_presence(
            &jid_a,
            &Presence {
                kind: None,
                show: Some(presence::Show(Show::Dnd)),
                status: None,
            },
        )
        .await?;

    cache.insert_user_activity(&jid_b, &None).await?;

    assert_eq!(
        cache.load_contacts().await?.into_iter().collect::<Vec<_>>(),
        vec![
            Contact {
                jid: jid_a.clone(),
                name: jid_a.to_string(),
                availability: Availability::DoNotDisturb,
                activity: None,
                groups: vec![String::from("")],
            },
            Contact {
                jid: jid_b.clone(),
                name: jid_b.to_string(),
                availability: Availability::Available,
                activity: None,
                groups: vec![String::from("")],
            }
        ]
    );

    let presence = xmpp_parsers::presence::Presence {
        from: Some(jid_a.clone().into()),
        to: None,
        id: None,
        // Test Type::None which xmpp_parsers cannot deserialize itself from a string
        type_: xmpp_parsers::presence::Type::None,
        show: Some(xmpp_parsers::presence::Show::Chat),
        statuses: Default::default(),
        priority: 0,
        payloads: vec![],
    };

    cache.insert_presence(&jid_a, &presence.into()).await?;

    assert_eq!(
        cache.load_contacts().await?.into_iter().collect::<Vec<_>>(),
        vec![
            Contact {
                jid: jid_a.clone(),
                name: jid_a.to_string(),
                availability: Availability::Available,
                activity: None,
                groups: vec![String::from("")],
            },
            Contact {
                jid: jid_b.clone(),
                name: jid_b.to_string(),
                availability: Availability::Available,
                activity: None,
                groups: vec![String::from("")],
            }
        ]
    );

    Ok(())
}

#[async_test]
async fn test_saves_draft() -> Result<()> {
    let cache = cache().await?;
    let jid_a = BareJid::from_str("a@prose.org").unwrap();
    let jid_b = BareJid::from_str("b@prose.org").unwrap();

    assert_eq!(cache.load_draft(&jid_a).await?, None);
    assert_eq!(cache.load_draft(&jid_b).await?, None);

    cache.save_draft(&jid_a, Some("Hello")).await?;
    cache.save_draft(&jid_b, Some("World")).await?;

    assert_eq!(cache.load_draft(&jid_a).await?, Some("Hello".to_string()));
    assert_eq!(cache.load_draft(&jid_b).await?, Some("World".to_string()));

    cache.save_draft(&jid_b, None).await?;

    assert_eq!(cache.load_draft(&jid_a).await?, Some("Hello".to_string()));
    assert_eq!(cache.load_draft(&jid_b).await?, None);

    Ok(())
}

#[async_test]
async fn test_can_insert_same_message_twice() -> Result<()> {
    let cache = cache().await?;

    let messages = [MessageLike {
        id: "1000".into(),
        stanza_id: None,
        target: None,
        to: BareJid::from_str("a@prose.org").unwrap(),
        from: BareJid::from_str("b@prose.org").unwrap(),
        timestamp: Utc
            .with_ymd_and_hms(2023, 04, 07, 16, 00, 00)
            .unwrap()
            .into(),
        payload: Payload::Message {
            body: String::from(""),
        },
        is_first_message: false,
    }];

    cache.insert_messages(&messages).await?;
    cache.insert_messages(&messages).await?;

    let loaded_messages = cache
        .load_messages_before(&BareJid::from_str("a@prose.org").unwrap(), None, 100)
        .await?;

    assert_eq!(
        Some(Page {
            items: messages.into_iter().collect(),
            is_complete: false
        }),
        loaded_messages
    );

    Ok(())
}

#[async_test]
async fn test_loads_message_with_emoji() -> Result<()> {
    let cache = cache().await?;

    let messages = [
        MessageLike {
            id: "1".into(),
            stanza_id: None,
            target: None,
            to: BareJid::from_str("a@prose.org").unwrap(),
            from: BareJid::from_str("b@prose.org").unwrap(),
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 07, 16, 00, 00)
                .unwrap()
                .into(),
            payload: Payload::Message {
                body: String::from("Hello World"),
            },
            is_first_message: false,
        },
        MessageLike {
            id: "2".into(),
            stanza_id: None,
            target: Some("1".into()),
            to: BareJid::from_str("a@prose.org").unwrap(),
            from: BareJid::from_str("b@prose.org").unwrap(),
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 07, 16, 00, 01)
                .unwrap()
                .into(),
            payload: Payload::Reaction {
                emojis: vec!["🍿".into()],
            },
            is_first_message: false,
        },
        MessageLike {
            id: "3".into(),
            stanza_id: None,
            target: Some("1".into()),
            to: BareJid::from_str("a@prose.org").unwrap(),
            from: BareJid::from_str("b@prose.org").unwrap(),
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 07, 16, 00, 02)
                .unwrap()
                .into(),
            payload: Payload::Reaction {
                emojis: vec!["🍿".into(), "📼".into()],
            },
            is_first_message: false,
        },
    ];

    cache.insert_messages(&messages).await?;

    let loaded_messages = cache
        .load_messages_before(&BareJid::from_str("a@prose.org").unwrap(), None, 100)
        .await?;

    assert_eq!(
        Some(Page {
            items: messages.into_iter().collect(),
            is_complete: false
        }),
        loaded_messages
    );

    Ok(())
}

#[async_test]
async fn test_load_messages_targeting() -> Result<()> {
    let cache = cache().await?;

    let messages = [
        MessageLike {
            id: "1000".into(),
            stanza_id: None,
            target: None,
            to: BareJid::from_str("a@prose.org").unwrap(),
            from: BareJid::from_str("b@prose.org").unwrap(),
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 07, 16, 00, 00)
                .unwrap()
                .into(),
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
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 07, 17, 00, 00)
                .unwrap()
                .into(),
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
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 17, 00, 00)
                .unwrap()
                .into(),
            payload: Payload::Retraction,
            is_first_message: false,
        },
        MessageLike {
            id: "2".into(),
            stanza_id: None,
            target: Some("1001".into()),
            to: BareJid::from_str("a@prose.org").unwrap(),
            from: BareJid::from_str("b@prose.org").unwrap(),
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 17, 00, 00)
                .unwrap()
                .into(),
            payload: Payload::Retraction,
            is_first_message: false,
        },
        MessageLike {
            id: "3".into(),
            stanza_id: None,
            target: Some("2000".into()),
            to: BareJid::from_str("a@prose.org").unwrap(),
            from: BareJid::from_str("b@prose.org").unwrap(),
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 18, 00, 00)
                .unwrap()
                .into(),
            payload: Payload::Retraction,
            is_first_message: false,
        },
        MessageLike {
            id: "4".into(),
            stanza_id: None,
            target: Some("1000".into()),
            to: BareJid::from_str("b@prose.org").unwrap(),
            from: BareJid::from_str("a@prose.org").unwrap(),
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 19, 00, 00)
                .unwrap()
                .into(),
            payload: Payload::Retraction,
            is_first_message: false,
        },
        MessageLike {
            id: "5".into(),
            stanza_id: None,
            target: Some("1000".into()),
            to: BareJid::from_str("a@prose.org").unwrap(),
            from: BareJid::from_str("c@prose.org").unwrap(),
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 20, 00, 00)
                .unwrap()
                .into(),
            payload: Payload::Retraction,
            is_first_message: false,
        },
        MessageLike {
            id: "6".into(),
            stanza_id: None,
            target: Some("1000".into()),
            to: BareJid::from_str("a@prose.org").unwrap(),
            from: BareJid::from_str("b@prose.org").unwrap(),
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 21, 00, 00)
                .unwrap()
                .into(),
            payload: Payload::Retraction,
            is_first_message: false,
        },
    ];

    cache.insert_messages(&messages).await?;

    assert_eq!(
        cache
            .load_messages_targeting(
                &BareJid::from_str("b@prose.org").unwrap(),
                &[message::Id::from("1000"), message::Id::from("1001")],
                &message::Id::from("1"),
                false
            )
            .await?,
        vec![
            messages[3].clone(),
            messages[5].clone(),
            messages[7].clone(),
        ]
    );

    assert_eq!(
        cache
            .load_messages_targeting(
                &BareJid::from_str("b@prose.org").unwrap(),
                &[message::Id::from("1000"), message::Id::from("1001")],
                None,
                true
            )
            .await?,
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

#[async_test]
async fn test_load_messages_before() -> Result<()> {
    let cache = cache().await?;

    let messages = [
        MessageLike {
            id: "1000".into(),
            stanza_id: Some("1".into()),
            target: None,
            to: BareJid::from_str("a@prose.org").unwrap(),
            from: BareJid::from_str("b@prose.org").unwrap(),
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 17, 00, 00)
                .unwrap()
                .into(),
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
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 18, 00, 00)
                .unwrap()
                .into(),
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
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 18, 00, 00)
                .unwrap()
                .into(),
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
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 19, 00, 00)
                .unwrap()
                .into(),
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
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 17, 00, 00)
                .unwrap()
                .into(),
            payload: Payload::Message {
                body: String::from("msg5"),
            },
            is_first_message: false,
        },
    ];

    cache.insert_messages(&messages).await?;

    // assert_eq!(
    //     cache
    //         .load_messages_before(
    //             &BareJid::from_str("b@prose.org").unwrap(),
    //             Some(&message::Id::from("3000")),
    //             100,
    //         )
    //         .await?,
    //     Some(Page {
    //         is_complete: true,
    //         items: vec![messages[0].clone(), messages[1].clone()]
    //     })
    // );
    //
    // assert_eq!(
    //     cache
    //         .load_messages_before(
    //             &BareJid::from_str("b@prose.org").unwrap(),
    //             Some(&message::Id::from("4000")),
    //             2,
    //         )
    //         .await?,
    //     Some(Page {
    //         is_complete: false,
    //         items: vec![messages[1].clone(), messages[2].clone()]
    //     })
    // );
    //
    // assert_eq!(
    //     cache
    //         .load_messages_before(
    //             &BareJid::from_str("b@prose.org").unwrap(),
    //             Some(&message::Id::from("1000")),
    //             100,
    //         )
    //         .await?,
    //     Some(Page {
    //         is_complete: true,
    //         items: vec![]
    //     })
    // );
    //
    // assert_eq!(
    //     cache
    //         .load_messages_before(
    //             &BareJid::from_str("c@prose.org").unwrap(),
    //             Some(&message::Id::from("5000")),
    //             100,
    //         )
    //         .await?,
    //     None
    // );

    assert_eq!(
        cache
            .load_messages_before(&BareJid::from_str("b@prose.org").unwrap(), None, 2,)
            .await?,
        Some(Page {
            is_complete: false,
            items: vec![messages[2].clone(), messages[3].clone()]
        })
    );

    // assert_eq!(
    //     cache
    //         .load_messages_before(&BareJid::from_str("d@prose.org").unwrap(), None, 100,)
    //         .await?,
    //     None
    // );

    Ok(())
}

#[async_test]
async fn test_load_messages_after() -> Result<()> {
    let cache = cache().await?;

    let messages = [
        MessageLike {
            id: "1000".into(),
            stanza_id: Some("1".into()),
            target: None,
            to: BareJid::from_str("a@prose.org").unwrap(),
            from: BareJid::from_str("b@prose.org").unwrap(),
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 17, 00, 00)
                .unwrap()
                .into(),
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
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 18, 00, 00)
                .unwrap()
                .into(),
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
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 18, 00, 00)
                .unwrap()
                .into(),
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
            timestamp: Utc
                .with_ymd_and_hms(2023, 04, 08, 18, 00, 00)
                .unwrap()
                .into(),
            payload: Payload::Message {
                body: String::from("msg4"),
            },
            is_first_message: false,
        },
    ];

    cache.insert_messages(&messages).await?;

    assert_eq!(
        cache
            .load_messages_after(
                &BareJid::from_str("b@prose.org").unwrap(),
                &"4000".into(),
                None,
            )
            .await?,
        vec![messages[1].clone()]
    );

    assert_eq!(
        cache
            .load_messages_after(
                &BareJid::from_str("b@prose.org").unwrap(),
                &"1000".into(),
                None,
            )
            .await?,
        vec![messages[1].clone(), messages[3].clone()]
    );

    assert_eq!(
        cache
            .load_messages_after(
                &BareJid::from_str("b@prose.org").unwrap(),
                &"1000".into(),
                Some(1),
            )
            .await?,
        vec![messages[3].clone()]
    );

    Ok(())
}
