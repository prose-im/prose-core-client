// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::app::event_handlers::{PubSubEventType, ServerEvent, SidebarBookmarkEvent};
use prose_core_client::domain::rooms::models::RoomSidebarState;
use prose_core_client::domain::shared::models::{MucId, UserId};
use prose_core_client::domain::sidebar::models::{Bookmark, BookmarkType};
use prose_core_client::test::parse_xml;
use prose_core_client::{muc_id, user_id};
use prose_proc_macros::mt_test;
use prose_xmpp::bare;

#[mt_test]
async fn test_added_or_updated_items() -> Result<()> {
    // Notification With Payload (https://xmpp.org/extensions/xep-0060.html#publisher-publish-success-withpayload)
    let events =
      parse_xml(
        r#"
        <message xmlns="jabber:client" from="user@prose.org" type="headline">
            <event xmlns='http://jabber.org/protocol/pubsub#event'>
                <items node="https://prose.org/protocol/bookmark">
                    <item id="pc@conference.prose.org">
                        <bookmark xmlns="https://prose.org/protocol/bookmark" name="Private Channel" jid="pc@conference.prose.org" type="private-channel" favorite="1" sidebar="1" />                
                    </item>
                    <item id="group@conference.prose.org">
                        <bookmark xmlns="https://prose.org/protocol/bookmark" name="Group" jid="group@conference.prose.org" type="group" />
                    </item>
                    <item id="user@prose.org">
                        <bookmark xmlns="https://prose.org/protocol/bookmark" name="Direct Message" jid="user@prose.org" type="dm" sidebar="1" />
                    </item>
                </items>
            </event>
        </message>
      "#,
      )
          .await?;

    assert_eq!(
        events,
        vec![ServerEvent::SidebarBookmark(SidebarBookmarkEvent {
            user_id: user_id!("user@prose.org"),
            r#type: PubSubEventType::AddedOrUpdated {
                items: vec![
                    Bookmark {
                        name: "Private Channel".to_string(),
                        jid: muc_id!("pc@conference.prose.org").into(),
                        r#type: BookmarkType::PrivateChannel,
                        sidebar_state: RoomSidebarState::Favorite
                    },
                    Bookmark {
                        name: "Group".to_string(),
                        jid: muc_id!("group@conference.prose.org").into(),
                        r#type: BookmarkType::Group,
                        sidebar_state: RoomSidebarState::NotInSidebar
                    },
                    Bookmark {
                        name: "Direct Message".to_string(),
                        jid: user_id!("user@prose.org").into(),
                        r#type: BookmarkType::DirectMessage,
                        sidebar_state: RoomSidebarState::InSidebar
                    }
                ]
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_deleted_items() -> Result<()> {
    // Delete And Notify (https://xmpp.org/extensions/xep-0060.html#example-119)
    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="user@prose.org" type="headline">
            <event xmlns='http://jabber.org/protocol/pubsub#event'>
                <items node="https://prose.org/protocol/bookmark">
                    <retract id="pc@conference.prose.org" />
                    <retract id="user@prose.org" />
                </items>
            </event>
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::SidebarBookmark(SidebarBookmarkEvent {
            user_id: user_id!("user@prose.org"),
            r#type: PubSubEventType::Deleted {
                ids: vec![bare!("pc@conference.prose.org"), bare!("user@prose.org"),]
            },
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_pubsub_node_purged() -> Result<()> {
    // Purge All Node Items (https://xmpp.org/extensions/xep-0060.html#example-166)
    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="user@prose.org" type="headline">
            <event xmlns='http://jabber.org/protocol/pubsub#event'>
                <purge node="https://prose.org/protocol/bookmark" />
            </event>
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::SidebarBookmark(SidebarBookmarkEvent {
            user_id: user_id!("user@prose.org"),
            r#type: PubSubEventType::Purged,
        })]
    );

    Ok(())
}

#[mt_test]
async fn test_pubsub_node_deleted() -> Result<()> {
    // Delete a node (https://xmpp.org/extensions/xep-0060.html#owner-delete)
    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="user@prose.org" type="headline">
            <event xmlns='http://jabber.org/protocol/pubsub#event'>
                <delete node="https://prose.org/protocol/bookmark" />
            </event>
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::SidebarBookmark(SidebarBookmarkEvent {
            user_id: user_id!("user@prose.org"),
            r#type: PubSubEventType::Purged,
        })]
    );

    Ok(())
}
