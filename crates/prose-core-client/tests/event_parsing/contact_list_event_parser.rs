// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::app::event_handlers::{ContactListEvent, ContactListEventType, ServerEvent};
use prose_core_client::domain::contacts::models::PresenceSubscription;
use prose_core_client::dtos::UserId;
use prose_core_client::test::parse_xml;
use prose_core_client::user_id;
use prose_proc_macros::mt_test;

#[mt_test]
async fn test_presence_subscription_request() -> Result<()> {
    // https://xmpp.org/rfcs/rfc6121.html#sub-request

    let events = parse_xml(
        r#"
        <presence xmlns="jabber:client" from="user@prose.org" type="subscribe" />
        "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::ContactList(ContactListEvent {
            contact_id: user_id!("user@prose.org"),
            r#type: ContactListEventType::PresenceSubscriptionRequested { nickname: None },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_contact_removed() -> Result<()> {
    let events = parse_xml(
        r#"
        <iq xmlns="jabber:client" id="request-id" type="set">
          <query xmlns="jabber:iq:roster" ver="1">
            <item jid="user@prose.org" subscription="remove" />
          </query>
        </iq>
        "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::ContactList(ContactListEvent {
            contact_id: user_id!("user@prose.org"),
            r#type: ContactListEventType::ContactRemoved,
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_presence_subscription_requested() -> Result<()> {
    let events = parse_xml(
        r#"
        <iq xmlns="jabber:client" id="request-id" type="set">
            <query xmlns="jabber:iq:roster" ver="1">
                <item ask="subscribe" jid="user@prose.org" subscription="none" />
            </query>
        </iq>
        "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::ContactList(ContactListEvent {
            contact_id: user_id!("user@prose.org"),
            r#type: ContactListEventType::ContactAddedOrPresenceSubscriptionUpdated {
                subscription: PresenceSubscription::Requested
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_mutual_presence_subscription() -> Result<()> {
    let events = parse_xml(
        r#"
        <iq xmlns="jabber:client" id="request-id" type="set">
          <query xmlns="jabber:iq:roster" ver="1">
            <item jid="user@prose.org" subscription="both">
              <group>Buddies</group>
              <group>Contacts</group>
            </item>
          </query>
        </iq>
        "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::ContactList(ContactListEvent {
            contact_id: user_id!("user@prose.org"),
            r#type: ContactListEventType::ContactAddedOrPresenceSubscriptionUpdated {
                subscription: PresenceSubscription::Mutual
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_contact_follows_us() -> Result<()> {
    let events = parse_xml(
        r#"
        <iq xmlns="jabber:client" id="request-id" type="set">
          <query xmlns="jabber:iq:roster" ver="1">
            <item jid="user@prose.org" subscription="from" />
          </query>
        </iq>
        "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::ContactList(ContactListEvent {
            contact_id: user_id!("user@prose.org"),
            r#type: ContactListEventType::ContactAddedOrPresenceSubscriptionUpdated {
                subscription: PresenceSubscription::TheyFollow
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_we_follow_contact() -> Result<()> {
    let events = parse_xml(
        r#"
        <iq xmlns="jabber:client" id="request-id" type="set">
          <query xmlns="jabber:iq:roster" ver="1">
            <item jid="user@prose.org" subscription="to" />
          </query>
        </iq>
        "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::ContactList(ContactListEvent {
            contact_id: user_id!("user@prose.org"),
            r#type: ContactListEventType::ContactAddedOrPresenceSubscriptionUpdated {
                subscription: PresenceSubscription::WeFollow
            },
        })]
    );

    Ok(())
}
