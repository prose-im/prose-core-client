// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use pretty_assertions::assert_eq;
use xmpp_parsers::roster::{Item as RosterItem, Subscription};

use prose_core_client::dtos::{Contact, Group, PresenceSubRequest, PresenceSubscription, UserId};
use prose_core_client::{user_id, ClientEvent};
use prose_proc_macros::mt_test;
use prose_xmpp::bare;
use prose_xmpp::stanza::{vcard4, VCard4};

use crate::{event, recv};

use super::helpers::{LoginStrategy, TestClient};

#[mt_test]
async fn test_presence_sub_request_name_cascade() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    // Magically we receive the vCard of one of the yet-to-be-confirmed contacts…
    client.receive_vcard(
        &user_id!("js@prose.org"),
        VCard4 {
            n: vec![vcard4::Name {
                surname: Some("Simone".to_string()),
                given: Some("June".to_string()),
                additional: None,
            }],
            ..Default::default()
        },
    );
    event!(
        client,
        ClientEvent::ContactChanged {
            ids: vec![user_id!("js@prose.org")]
        }
    );
    client.receive_next().await;

    recv!(
        client,
        r#"
        <presence xmlns="jabber:client" from="jane_doe@prose.org" to="{{USER_RESOURCE_ID}}" type="subscribe" xml:lang="en" />
        "#
    );
    event!(client, ClientEvent::PresenceSubRequestsChanged);
    client.receive_next().await;

    recv!(
        client,
        r#"
        <presence xmlns="jabber:client" from="j_schmoe@prose.org" to="{{USER_RESOURCE_ID}}" type="subscribe" xml:lang="en">
            <nick xmlns="http://jabber.org/protocol/nick">John Schmoe</nick>
        </presence>
        "#
    );
    event!(client, ClientEvent::PresenceSubRequestsChanged);
    event!(
        client,
        ClientEvent::ContactChanged {
            ids: vec![user_id!("j_schmoe@prose.org")]
        }
    );
    client.receive_next().await;

    recv!(
        client,
        r#"
        <presence xmlns="jabber:client" from="js@prose.org" to="{{USER_RESOURCE_ID}}" type="subscribe" xml:lang="en">
            <nick xmlns="http://jabber.org/protocol/nick">J. S.</nick>
        </presence>
        "#
    );
    event!(client, ClientEvent::PresenceSubRequestsChanged);
    event!(
        client,
        ClientEvent::ContactChanged {
            ids: vec![user_id!("js@prose.org")]
        }
    );
    client.receive_next().await;

    let requests = client.contact_list.load_presence_sub_requests().await?;

    assert_eq!(
        vec![
            PresenceSubRequest {
                id: user_id!("jane_doe@prose.org").into(),
                name: "Jane Doe".to_string(),
                user_id: user_id!("jane_doe@prose.org"),
            },
            PresenceSubRequest {
                id: user_id!("j_schmoe@prose.org").into(),
                name: "John Schmoe".to_string(),
                user_id: user_id!("j_schmoe@prose.org"),
            },
            PresenceSubRequest {
                id: user_id!("js@prose.org").into(),
                name: "June Simone".to_string(),
                user_id: user_id!("js@prose.org"),
            }
        ],
        requests
    );

    Ok(())
}

#[mt_test]
async fn test_contact_list_name_cascade() -> Result<()> {
    let client = TestClient::new().await;

    let strategy = LoginStrategy::default().with_roster_items([
        RosterItem {
            jid: bare!("user_a@prose.org"),
            name: None,
            subscription: Subscription::Both,
            ask: Default::default(),
            groups: vec![],
        },
        RosterItem {
            jid: bare!("b@prose.org"),
            name: Some("Susan Doe".to_string()),
            subscription: Subscription::Both,
            ask: Default::default(),
            groups: vec![],
        },
        RosterItem {
            jid: bare!("c@example.com"),
            name: Some("User C".to_string()),
            subscription: Subscription::Both,
            ask: Default::default(),
            groups: vec![],
        },
    ]);

    client
        .expect_login_with_strategy(user_id!("user@prose.org"), "secret", strategy)
        .await?;

    client.receive_vcard(
        &user_id!("c@example.com"),
        VCard4 {
            n: vec![vcard4::Name {
                surname: Some("Shmoe".to_string()),
                given: Some("Jimmy".to_string()),
                additional: None,
            }],
            ..Default::default()
        },
    );
    event!(
        client,
        ClientEvent::ContactChanged {
            ids: vec![user_id!("c@example.com")]
        }
    );
    client.receive_next().await;

    let contacts = client.contact_list.load_contacts().await?;

    assert_eq!(
        vec![
            Contact {
                id: user_id!("user_a@prose.org"),
                name: "User A".to_string(),
                full_name: None,
                availability: Default::default(),
                status: None,
                group: Group::Team,
                presence_subscription: PresenceSubscription::Mutual,
            },
            Contact {
                id: user_id!("b@prose.org"),
                name: "Susan Doe".to_string(),
                full_name: None,
                availability: Default::default(),
                status: None,
                group: Group::Team,
                presence_subscription: PresenceSubscription::Mutual,
            },
            Contact {
                id: user_id!("c@example.com"),
                name: "Jimmy Shmoe".to_string(),
                full_name: Some("Jimmy Shmoe".to_string()),
                availability: Default::default(),
                status: None,
                group: Group::Other,
                presence_subscription: PresenceSubscription::Mutual,
            }
        ],
        contacts
    );

    Ok(())
}
