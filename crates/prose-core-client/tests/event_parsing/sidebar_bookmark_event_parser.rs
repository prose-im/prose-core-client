// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::app::event_handlers::{ServerEvent, SidebarBookmarkEvent};
use prose_core_client::domain::shared::models::RoomId;
use prose_core_client::domain::sidebar::models::{Bookmark, BookmarkType};
use prose_core_client::room_id;
use prose_core_client::test::parse_xml;
use prose_proc_macros::mt_test;

#[mt_test]
async fn test_added_or_updated_items() -> Result<()> {
    // Notification With Payload (https://xmpp.org/extensions/xep-0060.html#publisher-publish-success-withpayload)
    let events =
      parse_xml(
        r#"
        <message xmlns="jabber:client" from="prose.org" type="headline">
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
        vec![ServerEvent::SidebarBookmark(
            SidebarBookmarkEvent::AddedOrUpdated {
                bookmarks: vec![
                    Bookmark {
                        name: "Private Channel".to_string(),
                        jid: room_id!("pc@conference.prose.org"),
                        r#type: BookmarkType::PrivateChannel,
                        is_favorite: true,
                        in_sidebar: true,
                    },
                    Bookmark {
                        name: "Group".to_string(),
                        jid: room_id!("group@conference.prose.org"),
                        r#type: BookmarkType::Group,
                        is_favorite: false,
                        in_sidebar: false,
                    },
                    Bookmark {
                        name: "Direct Message".to_string(),
                        jid: room_id!("user@prose.org"),
                        r#type: BookmarkType::DirectMessage,
                        is_favorite: false,
                        in_sidebar: true,
                    }
                ]
            }
        )]
    );

    Ok(())
}

#[mt_test]
async fn test_deleted_items() -> Result<()> {
    // Delete And Notify (https://xmpp.org/extensions/xep-0060.html#example-119)
    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="prose.org" type="headline">
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
        vec![ServerEvent::SidebarBookmark(
            SidebarBookmarkEvent::Deleted {
                ids: vec![
                    room_id!("pc@conference.prose.org"),
                    room_id!("user@prose.org"),
                ]
            }
        )]
    );

    Ok(())
}

#[mt_test]
async fn test_pubsub_node_purged() -> Result<()> {
    // Purge All Node Items (https://xmpp.org/extensions/xep-0060.html#example-166)
    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="prose.org" type="headline">
            <event xmlns='http://jabber.org/protocol/pubsub#event'>
                <purge node="https://prose.org/protocol/bookmark" />
            </event>
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::SidebarBookmark(SidebarBookmarkEvent::Purged)]
    );

    Ok(())
}

#[mt_test]
async fn test_pubsub_node_deleted() -> Result<()> {
    // Delete a node (https://xmpp.org/extensions/xep-0060.html#owner-delete)
    let events = parse_xml(
        r#"
        <message xmlns="jabber:client" from="prose.org" type="headline">
            <event xmlns='http://jabber.org/protocol/pubsub#event'>
                <delete node="https://prose.org/protocol/bookmark" />
            </event>
        </message>
      "#,
    )
    .await?;

    assert_eq!(
        events,
        vec![ServerEvent::SidebarBookmark(SidebarBookmarkEvent::Purged)]
    );

    Ok(())
}
