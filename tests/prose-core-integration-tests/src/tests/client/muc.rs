// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::BareJid;
use pretty_assertions::assert_eq;

use prose_core_client::domain::shared::models::AnonOccupantId;
use prose_core_client::domain::sidebar::models::BookmarkType;
use prose_core_client::dtos::{
    MucId, ParticipantId, SendMessageRequest, SendMessageRequestBody, UserId,
};
use prose_core_client::{muc_id, user_id, ClientEvent, ClientRoomEventType};
use prose_proc_macros::mt_test;

use crate::{event, recv, room_event, send};

use super::helpers::TestClient;

#[mt_test]
async fn test_joins_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    client
        .join_room(muc_id!("room@conference.prose.org"), "anon-id")
        .await?;

    Ok(())
}

#[mt_test]
async fn test_creates_public_channel() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("org.prose.channel.short-id-2@conference.prose.org");
    let occupant_id = client.build_occupant_id(&room_id);
    let room_name = "My Public Channel";

    client.push_ctx([
        (
            "MUC_SERVICE_ID",
            BareJid::from_parts(None, &room_id.as_ref().domain()).to_string(),
        ),
        ("OCCUPANT_ID", occupant_id.to_string()),
        ("ROOM_ID", room_id.to_string()),
        ("ROOM_NAME", room_name.into()),
    ]);

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{MUC_SERVICE_ID}}" type="get">
          <query xmlns="http://jabber.org/protocol/disco#items" />
        </iq>
        "#
    );

    recv!(
        client,
        r#"
        <iq xmlns="jabber:client" from="{{MUC_SERVICE_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/disco#items">
            <item jid="general@groups.prose.org" name="general" />
          </query>
        </iq>
        "#
    );

    send!(
        client,
        r#"
        <presence xmlns='jabber:client' to="{{OCCUPANT_ID}}">
            <show>chat</show>
            <x xmlns='http://jabber.org/protocol/muc'>
              <history maxstanzas="0" />
            </x>
            <c xmlns='http://jabber.org/protocol/caps' hash="sha-1" node="https://prose.org" ver="{{CAPS_HASH}}"/>
            <nick xmlns="http://jabber.org/protocol/nick">Jane Doe</nick>
        </presence>
        "#
    );

    recv!(
        client,
        r#"
        <presence xmlns="jabber:client" from="{{OCCUPANT_ID}}" xml:lang="en">
          <show>chat</show>
          <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="https://prose.org" ver="{{CAPS_HASH}}" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{ANON_OCCUPANT_ID}}" />
          <x xmlns="http://jabber.org/protocol/muc#user">
            <status code="201" />
            <item affiliation="owner" jid="{{USER_RESOURCE_ID}}" role="moderator" />
            <status code="110" />
          </x>
        </presence>
        "#
    );

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{ROOM_ID}}" to="{{USER_RESOURCE_ID}}" type="groupchat">
          <subject />
        </message>        
        "#
    );

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{ROOM_ID}}" type="get">
            <query xmlns="http://jabber.org/protocol/muc#owner" />
        </iq>
        "#
    );

    recv!(
        client,
        r#"
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/muc#owner">
            <x xmlns="jabber:x:data" type="form">
              <title>Configuration for {{ROOM_ID}}</title>
              <instructions>Complete and submit this form to configure the room.</instructions>
              <field type="hidden" var="FORM_TYPE">
                <value>http://jabber.org/protocol/muc#roomconfig</value>
              </field>
              <field type="fixed">
                <value>Room information</value>
              </field>
              <field label="Title" type="text-single" var="muc#roomconfig_roomname" />
              <field label="Description" type="text-single" var="muc#roomconfig_roomdesc">
                <desc>A brief description of the room</desc>
                <value />
              </field>
              <field label="Language tag for room (e.g. &apos;en&apos;, &apos;de&apos;, &apos;fr&apos; etc.)" type="text-single" var="muc#roomconfig_lang">
                <desc>Indicate the primary language spoken in this room</desc>
                <validate xmlns="http://jabber.org/protocol/xdata-validate" datatype="xs:language" />
                <value>en</value>
              </field>
              <field label="Persistent (room should remain even when it is empty)" type="boolean" var="muc#roomconfig_persistentroom">
                <desc>Rooms are automatically deleted when they are empty, unless this option is enabled</desc>
              </field>
              <field label="Include room information in public lists" type="boolean" var="muc#roomconfig_publicroom">
                <desc>Enable this to allow people to find the room</desc>
                <value>0</value>
              </field>
              <field type="fixed">
                <value>Access to the room</value>
              </field>
              <field label="Password" type="text-private" var="muc#roomconfig_roomsecret">
                <value />
              </field>
              <field label="Only allow members to join" type="boolean" var="muc#roomconfig_membersonly">
                <desc>Enable this to only allow access for room owners, admins and members</desc>
              </field>
              <field label="Allow members to invite new members" type="boolean" var="{http://prosody.im/protocol/muc}roomconfig_allowmemberinvites" />
              <field type="fixed">
                <value>Permissions in the room</value>
              </field>
              <field label="Allow anyone to set the room&apos;s subject" type="boolean" var="muc#roomconfig_changesubject">
                <desc>Choose whether anyone, or only moderators, may set the room's subject</desc>
              </field>
              <field label="Moderated (require permission to speak)" type="boolean" var="muc#roomconfig_moderatedroom">
                <desc>In moderated rooms occupants must be given permission to speak by a room moderator</desc>
              </field>
              <field label="Addresses (JIDs) of room occupants may be viewed by:" type="list-single" var="muc#roomconfig_whois">
                <option label="Moderators only">
                  <value>moderators</value>
                </option>
                <option label="Anyone">
                  <value>anyone</value>
                </option>
                <value>moderators</value>
              </field>
              <field type="fixed">
                <value>Other options</value>
              </field>
              <field label="Maximum number of history messages returned by room" type="text-single" var="muc#roomconfig_historylength">
                <desc>Specify the maximum number of previous messages that should be sent to users when they join the room</desc>
                <validate xmlns="http://jabber.org/protocol/xdata-validate" datatype="xs:integer" />
                <value>20</value>
              </field>
              <field label="Default number of history messages returned by room" type="text-single" var="muc#roomconfig_defaulthistorymessages">
                <desc>Specify the number of previous messages sent to new users when they join the room</desc>
                <validate xmlns="http://jabber.org/protocol/xdata-validate" datatype="xs:integer" />
                <value>20</value>
              </field>
              <field label="Only show participants with roles:" type="list-multi" var="muc#roomconfig_presencebroadcast">
                <option label="none">
                  <value>none</value>
                </option>
                <option label="visitor">
                  <value>visitor</value>
                </option>
                <option label="participant">
                  <value>participant</value>
                </option>
                <option label="moderator">
                  <value>moderator</value>
                </option>
                <value>moderator</value>
                <value>visitor</value>
                <value>participant</value>
              </field>
              <field label="Archive chat on server" type="boolean" var="muc#roomconfig_enablearchiving">
                <value>1</value>
              </field>
            </x>
          </query>
        </iq>
        "#
    );

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{ROOM_ID}}" type="set">
          <query xmlns="http://jabber.org/protocol/muc#owner">
            <x xmlns="jabber:x:data" type="submit">
              <field type="hidden" var="FORM_TYPE">
                <value>http://jabber.org/protocol/muc#roomconfig</value>
              </field>
              <field label="Title" var="muc#roomconfig_roomname">
                <value>{{ROOM_NAME}}</value>
              </field>
              <field label="Description" var="muc#roomconfig_roomdesc">
                <value />
              </field>
              <field label="Language tag for room (e.g. &apos;en&apos;, &apos;de&apos;, &apos;fr&apos; etc.)" var="muc#roomconfig_lang">
                <value>en</value>
              </field>
              <field label="Persistent (room should remain even when it is empty)" type="boolean" var="muc#roomconfig_persistentroom">
                <value>true</value>
              </field>
              <field label="Include room information in public lists" type="boolean" var="muc#roomconfig_publicroom">
                <value>true</value>
              </field>
              <field label="Password" type="text-private" var="muc#roomconfig_roomsecret">
                <value />
              </field>
              <field label="Only allow members to join" type="boolean" var="muc#roomconfig_membersonly">
                <value>false</value>
              </field>
              <field label="Allow members to invite new members" type="boolean" var="{http://prosody.im/protocol/muc}roomconfig_allowmemberinvites">
                <value>true</value>
              </field>
              <field label="Allow anyone to set the room&apos;s subject" type="boolean" var="muc#roomconfig_changesubject">
                <value>true</value>
              </field>
              <field label="Moderated (require permission to speak)" type="boolean" var="muc#roomconfig_moderatedroom">
                <value>false</value>
              </field>
              <field label="Addresses (JIDs) of room occupants may be viewed by:" type="list-single" var="muc#roomconfig_whois">
                <option label="Moderators only">
                  <value>moderators</value>
                </option>
                <option label="Anyone">
                  <value>anyone</value>
                </option>
                <value>anyone</value>
              </field>
              <field label="Maximum number of history messages returned by room" var="muc#roomconfig_historylength">
                <value>20</value>
              </field>
              <field label="Default number of history messages returned by room" var="muc#roomconfig_defaulthistorymessages">
                <value>0</value>
              </field>
              <field label="Only show participants with roles:" type="list-multi" var="muc#roomconfig_presencebroadcast">
                <option label="none">
                  <value>none</value>
                </option>
                <option label="visitor">
                  <value>visitor</value>
                </option>
                <option label="participant">
                  <value>participant</value>
                </option>
                <option label="moderator">
                  <value>moderator</value>
                </option>
                <value>moderator</value>
                <value>participant</value>
                <value>visitor</value>
              </field>
              <field label="Archive chat on server" type="boolean" var="muc#roomconfig_enablearchiving">
                <value>1</value>
              </field>
            </x>
          </query>
        </iq>
        "#
    );

    recv!(
        client,
        r#"
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result" />
        "#
    );

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{ROOM_ID}}" to="{{USER_RESOURCE_ID}}" type="groupchat">
          <x xmlns="http://jabber.org/protocol/muc#user">
            <status code="170" />
            <status code="172" />
            <status code="104" />
          </x>
        </message>
        "#
    );

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{ROOM_ID}}" type="get">
          <query xmlns="http://jabber.org/protocol/disco#info" />
        </iq>
        "#
    );

    recv!(
        client,
        r#"
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/disco#info">
            <feature var="muc_unsecured" />
            <identity category="conference" name="{{ROOM_NAME}}" type="text" />
            <feature var="http://jabber.org/protocol/muc#request" />
            <feature var="http://jabber.org/protocol/muc" />
            <feature var="http://jabber.org/protocol/muc#stable_id" />
            <feature var="http://jabber.org/protocol/muc#self-ping-optimization" />
            <feature var="muc_unmoderated" />
            <feature var="muc_persistent" />
            <feature var="muc_open" />
            <feature var="jabber:iq:register" />
            <feature var="urn:xmpp:mam:2" />
            <feature var="urn:xmpp:mam:2#extended" />
            <feature var="urn:xmpp:sid:0" />
            <feature var="urn:xmpp:occupant-id:0" />
            <feature var="muc_public" />
            <feature var="muc_nonanonymous" />
            <x xmlns="jabber:x:data" type="result">
              <field type="hidden" var="FORM_TYPE">
                <value>http://jabber.org/protocol/muc#roominfo</value>
              </field>
              <field label="Title" type="text-single" var="muc#roomconfig_roomname">
                <value>{{ROOM_NAME}}</value>
              </field>
              <field type="text-single" var="muc#roominfo_lang">
                <value>en</value>
              </field>
              <field label="Description" type="text-single" var="muc#roominfo_description">
                <value />
              </field>
              <field type="boolean" var="muc#roomconfig_changesubject">
                <value>1</value>
              </field>
              <field label="Number of occupants" type="text-single" var="muc#roominfo_occupants">
                <value>1</value>
              </field>
              <field label="Allow members to invite new members" type="boolean" var="{http://prosody.im/protocol/muc}roomconfig_allowmemberinvites">
                <value>1</value>
              </field>
              <field label="Allow users to invite other users" type="boolean" var="muc#roomconfig_allowinvites">
                <value>1</value>
              </field>
            </x>
          </query>
        </iq>
        "#
    );

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{ROOM_ID}}" type="get">
          <query xmlns="http://jabber.org/protocol/muc#admin">
            <item xmlns="http://jabber.org/protocol/muc#user" affiliation="owner" />
          </query>
        </iq>
        "#
    );
    recv!(
        client,
        r#"
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/muc#admin">
            <item affiliation="owner" jid="{{USER_ID}}" />
          </query>
        </iq>
        "#
    );

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{ROOM_ID}}" type="get">
          <query xmlns="http://jabber.org/protocol/muc#admin">
            <item xmlns="http://jabber.org/protocol/muc#user" affiliation="member" />
          </query>
        </iq>
        "#
    );
    recv!(
        client,
        r#"
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/muc#admin" />
        </iq>
        "#
    );

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{ROOM_ID}}" type="get">
          <query xmlns="http://jabber.org/protocol/muc#admin">
            <item xmlns="http://jabber.org/protocol/muc#user" affiliation="admin" />
          </query>
        </iq>
        "#
    );
    recv!(
        client,
        r#"
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/muc#admin" />
        </iq>
        "#
    );

    client.pop_ctx();

    client.expect_load_synced_room_settings(room_id.clone(), None);
    client.expect_muc_catchup(&room_id);
    client.expect_set_bookmark(room_id.clone(), room_name, BookmarkType::PublicChannel);

    event!(client, ClientEvent::SidebarChanged);

    client
        .rooms
        .create_room_for_public_channel(room_name)
        .await?;

    Ok(())
}

#[mt_test]
async fn test_receives_chat_states() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@conference.prose.org");
    let occupant_id = client.build_occupant_id(&room_id);

    client.join_room(room_id.clone(), "anon-id").await?;

    let room = client.get_room(room_id.clone()).await.to_generic_room();

    client.push_ctx([
        ("OCCUPANT_ID", occupant_id.to_string()),
        ("OTHER_OCCUPANT_ID", format!("{room_id}/their-nick")),
        ("OTHER_ANON_OCCUPANT_ID", "their-anon-id".to_string()),
        (
            "OTHER_USER_RESOURCE_ID",
            "user2@prose.org/resource".to_string(),
        ),
        ("ROOM_ID", room_id.to_string()),
        ("STANZA_ID", "stanza-id".to_string()),
    ]);

    recv!(
        client,
        r#"
        <presence xmlns="jabber:client" from="{{OTHER_OCCUPANT_ID}}">
            <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="http://conversations.im" ver="VaFH3zLveT77pOMcOwsKdlw2IPE=" />
            <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{OTHER_ANON_OCCUPANT_ID}}" />
            <x xmlns="http://jabber.org/protocol/muc#user">
                <item affiliation="member" jid="{{OTHER_USER_RESOURCE_ID}}" role="participant" />
            </x>
        </presence>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::ParticipantsChanged
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::ParticipantsChanged
    );

    client.receive_next().await;

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{OTHER_OCCUPANT_ID}}" id="message-id-1" to="{{USER_RESOURCE_ID}}" type="groupchat" xml:lang="en">
            <composing xmlns="http://jabber.org/protocol/chatstates" />
            <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{OTHER_ANON_OCCUPANT_ID}}" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::ComposingUsersChanged
    );

    client.receive_next().await;

    let composing_users = room.load_composing_users().await?;
    assert_eq!(1, composing_users.len());
    assert_eq!(
        ParticipantId::Occupant(room_id.occupant_id_with_nickname("their-nick")?),
        composing_users[0].id
    );

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{OTHER_OCCUPANT_ID}}" id="message-id-2" to="{{USER_RESOURCE_ID}}" type="groupchat" xml:lang="en">
          <body>Hello World</body>
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{OTHER_ANON_OCCUPANT_ID}}" />
          <stanza-id xmlns="urn:xmpp:sid:0" by="{{ROOM_ID}}" id="{{STANZA_ID}}" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::ComposingUsersChanged
    );
    event!(client, ClientEvent::SidebarChanged);

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesAppended {
            message_ids: vec!["message-id-2".into()]
        }
    );

    client.receive_next().await;

    let composing_users = room.load_composing_users().await?;
    assert!(composing_users.is_empty());

    let messages = room
        .load_messages_with_ids(&["message-id-2".into()])
        .await?;
    assert_eq!(1, messages.len());
    assert_eq!(Some("stanza-id".into()), messages[0].stanza_id);

    Ok(())
}

#[mt_test]
async fn test_sends_and_updates_message_to_muc_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@conference.prose.org");
    let occupant_id = client.build_occupant_id(&room_id);
    let anon_occupant_id = AnonOccupantId::from("anon-occupant-id");

    client
        .join_room(room_id.clone(), anon_occupant_id.clone())
        .await?;

    client.push_ctx([
        ("OCCUPANT_ID", occupant_id.to_string()),
        ("ROOM_ID", room_id.to_string()),
        ("ANON_OCCUPANT_ID", anon_occupant_id.to_string()),
    ]);

    let message_id = client.get_next_id();

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="{{ROOM_ID}}" type="groupchat">
          <body>Hello</body>
          <content xmlns="urn:xmpp:content" type="text/markdown">Hello</content>
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesAppended {
            message_ids: vec![message_id.clone().into()]
        }
    );

    let room = client.get_room(room_id.clone()).await.to_generic_room();
    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello".into(),
        }),
        attachments: vec![],
    })
    .await?;

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{OCCUPANT_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="groupchat" xml:lang="en">
          <body>Hello</body>
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{ANON_OCCUPANT_ID}}" />
          <stanza-id xmlns="urn:xmpp:sid:0" by="{{ROOM_ID}}" id="opZdWmO7r50ee_aGKnWvBMbK" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesUpdated {
            message_ids: vec![message_id.clone().into()]
        }
    );

    client.receive_next().await;

    let messages = room
        .load_messages_with_ids(&[message_id.clone().into()])
        .await?;
    assert_eq!(1, messages.len());
    assert_eq!("<p>Hello</p>", messages[0].body.html.as_ref());
    assert_eq!(
        Some("opZdWmO7r50ee_aGKnWvBMbK".into()),
        messages[0].stanza_id,
    );

    client.push_ctx([("INITIAL_MESSAGE_ID", message_id.to_string())]);

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="{{ROOM_ID}}" type="groupchat">
          <body>Hello World</body>
          <content xmlns="urn:xmpp:content" type="text/markdown">Hello World</content>
          <replace xmlns="urn:xmpp:message-correct:0" id="{{INITIAL_MESSAGE_ID}}" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesUpdated {
            message_ids: vec![message_id.clone().into()]
        }
    );

    room.update_message(
        message_id.clone().into(),
        SendMessageRequest {
            body: Some(SendMessageRequestBody {
                text: "Hello World".into(),
            }),
            attachments: vec![],
        },
    )
    .await?;

    let messages = room
        .load_messages_with_ids(&[message_id.clone().into()])
        .await?;
    assert_eq!(1, messages.len());
    assert_eq!("<p>Hello World</p>", messages[0].body.html.as_ref());

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{OCCUPANT_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="groupchat" xml:lang="en">
          <body>Hello World</body>
          <replace xmlns="urn:xmpp:message-correct:0" id="{{INITIAL_MESSAGE_ID}}" />
          <store xmlns="urn:xmpp:hints" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{ANON_OCCUPANT_ID}}" />
          <stanza-id xmlns="urn:xmpp:sid:0" by="{{ROOM_ID}}" id="907z40xwIIuX4b1YH5jRv1ko" />
        </message>
        "#
    );

    room_event!(
        client,
        room_id.clone(),
        ClientRoomEventType::MessagesUpdated {
            message_ids: vec![message_id.clone().into()]
        }
    );

    client.receive_next().await;

    let messages = room
        .load_messages_with_ids(&[message_id.clone().into()])
        .await?;
    assert_eq!(1, messages.len());
    assert_eq!("<p>Hello World</p>", messages[0].body.html.as_ref());

    client.pop_ctx();
    client.pop_ctx();

    Ok(())
}
