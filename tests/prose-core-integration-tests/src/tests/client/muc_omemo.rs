// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use minidom::Element;

use prose_core_client::domain::shared::models::AccountId;
use prose_core_client::domain::sidebar::models::BookmarkType;
use prose_core_client::dtos::{
    DeviceBundle, MucId, RoomId, SendMessageRequest, SendMessageRequestBody, UserId,
};
use prose_core_client::{account_id, muc_id, user_id, ClientEvent, ClientRoomEventType};
use prose_proc_macros::mt_test;

use crate::tests::client::helpers::{TestClient, TestDeviceBundle};
use crate::{event, recv, room_event, send};

#[mt_test]
async fn test_decrypts_message_from_private_nonanonymous_muc_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@prose.org");
    let room_name = "omemo-private-channel";
    let occupant_id = client.build_occupant_id(&room_id);

    client.push_ctx([
        ("OCCUPANT_ID", occupant_id.to_string()),
        ("ANON_OCCUPANT_ID", "some-weird-id".to_string()),
        ("ROOM_ID", room_id.to_string()),
        ("ROOM_NAME", room_name.to_string()),
    ]);

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
        <presence xmlns="jabber:client" from="{{ROOM_ID}}/nick">
            <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="http://conversations.im" ver="VaFH3zLveT77pOMcOwsKdlw2IPE=" />
            <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{ANON_OCCUPANT_ID}}" />
            <x xmlns="http://jabber.org/protocol/muc#user">
                <item affiliation="member" jid="user2@prose.org/resource" role="participant" />
            </x>
        </presence>
        "#
    );

    recv!(
        client,
        r#"
        <presence xmlns="jabber:client" from="{{OCCUPANT_ID}}" xml:lang="en">
          <show>chat</show>
          <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="https://prose.org" ver="{{CAPS_HASH}}" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="occupant-id" />
          <x xmlns="http://jabber.org/protocol/muc#user">
            <status code="100" />
            <item affiliation="owner" jid="{{USER_RESOURCE_ID}}" role="moderator" />
            <status code="110" />
          </x>
        </presence>
        "#
    );
    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{ROOM_ID}}" type="groupchat">
          <subject />
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
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/disco#info">
            <feature var="muc_unsecured" />
            <feature var="muc_nonanonymous" />
            <identity category="conference" name="{{ROOM_NAME}}" type="text" />
            <feature var="muc_persistent" />
            <feature var="http://jabber.org/protocol/muc#request" />
            <feature var="http://jabber.org/protocol/muc" />
            <feature var="http://jabber.org/protocol/muc#stable_id" />
            <feature var="http://jabber.org/protocol/muc#self-ping-optimization" />
            <feature var="muc_unmoderated" />
            <feature var="muc_membersonly" />
            <feature var="urn:xmpp:mam:2" />
            <feature var="urn:xmpp:mam:2#extended" />
            <feature var="urn:xmpp:sid:0" />
            <feature var="urn:xmpp:occupant-id:0" />
            <feature var="jabber:iq:register" />
            <feature var="muc_hidden" />
            <x xmlns="jabber:x:data" type="result">
              <field type="hidden" var="FORM_TYPE">
                <value>http://jabber.org/protocol/muc#roominfo</value>
              </field>
              <field type="boolean" var="muc#roomconfig_changesubject">
                <value>1</value>
              </field>
              <field label="Title" type="text-single" var="muc#roomconfig_roomname">
                <value>{{ROOM_NAME}}</value>
              </field>
              <field label="Allow members to invite new members" type="boolean" var="{http://prosody.im/protocol/muc}roomconfig_allowmemberinvites">
                <value>1</value>
              </field>
              <field label="Allow users to invite other users" type="boolean" var="muc#roomconfig_allowinvites">
                <value>1</value>
              </field>
              <field type="text-single" var="muc#roominfo_lang">
                <value>en</value>
              </field>
              <field label="Number of occupants" type="text-single" var="muc#roominfo_occupants">
                <value>3</value>
              </field>
              <field label="Description" type="text-single" var="muc#roominfo_description">
                <value />
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
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/muc#admin">
            <item affiliation="owner" jid="user1@prose.org" />
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
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/muc#admin">
            <item affiliation="member" jid="user2@prose.org" />
            <item affiliation="member" jid="user3@prose.org" />
          </query>
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
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/muc#admin" />
        </iq>
        "#
    );

    client.expect_load_synced_room_settings(room_id.clone(), None);
    client.expect_muc_catchup(&room_id);
    client.expect_set_bookmark(room_id.clone(), room_name, BookmarkType::PrivateChannel);

    event!(client, ClientEvent::SidebarChanged);

    client.rooms.join_room(&room_id, None).await?;

    let room = client
        .get_room(RoomId::Muc(room_id.clone()))
        .await
        .to_generic_room();

    assert!(client
        .user_data
        .load_user_device_infos(&user_id!("user2@prose.org"))
        .await?
        .is_empty());

    let service = TestClient::their_encryption_domain_service(user_id!("user2@prose.org")).await;
    let encrypted_payload = service
        .encrypt_message(
            vec![user_id!("user@prose.org")],
            "Can you read this?".to_string(),
        )
        .await?;

    client.push_ctx([(
        "ENCRYPTED_PAYLOAD",
        String::from(&Element::from(xmpp_parsers::legacy_omemo::Encrypted::from(
            encrypted_payload,
        ))),
    )]);

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{ROOM_ID}}/nick" id="my-message-id" to="{{USER_RESOURCE_ID}}" type="groupchat">
          <body>[This message is OMEMO encrypted]</body>
          {{ENCRYPTED_PAYLOAD}}
          <encryption xmlns="urn:xmpp:eme:0" name="OMEMO" namespace="eu.siacs.conversations.axolotl" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{ANON_OCCUPANT_ID}}" />
        </message>
        "#
    );

    // The bundle should be published with a fresh PreKey…
    let bundle_xml = TestClient::initial_device_bundle_xml().replace(
        r#"<preKeyPublic preKeyId="1">BW5hMOrNOjAiWAex/RebnNDAq4vFVz30wLGFhBSAdyoy</preKeyPublic>"#,
        r#"<preKeyPublic preKeyId="1">BTQ9Qr1iZH0bYjwm34NOaKoc3g2bCKMzsqyeNihNgaUx</preKeyPublic>"#,
    );
    assert_ne!(bundle_xml, TestClient::initial_device_bundle_xml());
    client.expect_publish_device_bundle(bundle_xml);

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" to="user2@prose.org" type="chat">
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="12345">
              <key rid="54321">NAohBTQ9Qr1iZH0bYjwm34NOaKoc3g2bCKMzsqyeNihNgaUxEAAYACIw7ZO7UG52jDS1YjzpynOptmzLu8URBehnmeuWPRP1YXbZ4NlaJMCoxcsKweRglZtbu6zdIVXext0=</key>
              <iv>AQAAAAAAAAACAAAA</iv>
            </header>
          </encrypted>
          <encryption xmlns="urn:xmpp:eme:0" name="OMEMO" namespace="eu.siacs.conversations.axolotl" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    let message_id = client.get_next_message_id();

    event!(client, ClientEvent::SidebarChanged);
    room_event!(
        client,
        room_id,
        ClientRoomEventType::MessagesAppended {
            message_ids: vec![message_id.clone()]
        }
    );

    client.receive_next().await;

    let messages = room.load_messages_with_ids(&[message_id]).await?;
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages.first().unwrap().body.html.as_ref(),
        "<p>Can you read this?</p>"
    );

    Ok(())
}

#[mt_test]
async fn test_encrypts_message_in_private_nonanonymous_muc_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@prose.org");
    let room_name = "omemo-private-channel";
    let occupant_id = client.build_occupant_id(&room_id);

    client.push_ctx([
        ("OCCUPANT_ID", occupant_id.to_string()),
        ("ANON_OCCUPANT_ID", "some-weird-id".to_string()),
        ("ROOM_ID", room_id.to_string()),
        ("ROOM_NAME", room_name.to_string()),
    ]);

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
        <presence xmlns="jabber:client" from="{{ROOM_ID}}/nick">
            <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="http://conversations.im" ver="VaFH3zLveT77pOMcOwsKdlw2IPE=" />
            <occupant-id xmlns="urn:xmpp:occupant-id:0" id="occupant-id" />
            <x xmlns="http://jabber.org/protocol/muc#user">
                <item affiliation="member" jid="user2@prose.org/resource" role="participant" />
            </x>
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
            <status code="100" />
            <item affiliation="owner" jid="{{USER_RESOURCE_ID}}" role="moderator" />
            <status code="110" />
          </x>
        </presence>
        "#
    );
    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{ROOM_ID}}" type="groupchat">
          <subject />
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
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/disco#info">
            <feature var="muc_unsecured" />
            <feature var="muc_nonanonymous" />
            <identity category="conference" name="{{ROOM_NAME}}" type="text" />
            <feature var="muc_persistent" />
            <feature var="http://jabber.org/protocol/muc#request" />
            <feature var="http://jabber.org/protocol/muc" />
            <feature var="http://jabber.org/protocol/muc#stable_id" />
            <feature var="http://jabber.org/protocol/muc#self-ping-optimization" />
            <feature var="muc_unmoderated" />
            <feature var="muc_membersonly" />
            <feature var="urn:xmpp:mam:2" />
            <feature var="urn:xmpp:mam:2#extended" />
            <feature var="urn:xmpp:sid:0" />
            <feature var="urn:xmpp:occupant-id:0" />
            <feature var="jabber:iq:register" />
            <feature var="muc_hidden" />
            <x xmlns="jabber:x:data" type="result">
              <field type="hidden" var="FORM_TYPE">
                <value>http://jabber.org/protocol/muc#roominfo</value>
              </field>
              <field type="boolean" var="muc#roomconfig_changesubject">
                <value>1</value>
              </field>
              <field label="Title" type="text-single" var="muc#roomconfig_roomname">
                <value>{{ROOM_NAME}}</value>
              </field>
              <field label="Allow members to invite new members" type="boolean" var="{http://prosody.im/protocol/muc}roomconfig_allowmemberinvites">
                <value>1</value>
              </field>
              <field label="Allow users to invite other users" type="boolean" var="muc#roomconfig_allowinvites">
                <value>1</value>
              </field>
              <field type="text-single" var="muc#roominfo_lang">
                <value>en</value>
              </field>
              <field label="Number of occupants" type="text-single" var="muc#roominfo_occupants">
                <value>3</value>
              </field>
              <field label="Description" type="text-single" var="muc#roominfo_description">
                <value />
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
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/muc#admin">
            <item affiliation="owner" jid="user1@prose.org" />
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
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/muc#admin">
            <item affiliation="member" jid="user2@prose.org" />
          </query>
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
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/muc#admin" />
        </iq>
        "#
    );

    client.expect_load_synced_room_settings(room_id.clone(), None);
    client.expect_muc_catchup(&room_id);
    client.expect_set_bookmark(room_id.clone(), room_name, BookmarkType::PrivateChannel);

    event!(client, ClientEvent::SidebarChanged);

    client.rooms.join_room(&room_id, None).await?;

    let room = client
        .get_room(RoomId::Muc(room_id.clone()))
        .await
        .to_generic_room();

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" type="set">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="https://prose.org/protocol/room_settings">
              <item id="{{ROOM_ID}}">
                <room-settings xmlns="https://prose.org/protocol/room_settings" room-id="muc:{{ROOM_ID}}">
                  <encryption type="omemo" />
                </room-settings>
              </item>
            </publish>
            <publish-options>
              <x xmlns="jabber:x:data" type="submit">
                <field type="hidden" var="FORM_TYPE">
                  <value>http://jabber.org/protocol/pubsub#publish-options</value>
                </field>
                <field type="boolean" var="pubsub#persist_items">
                  <value>true</value>
                </field>
                <field var="pubsub#access_model">
                  <value>whitelist</value>
                </field>
                <field var="pubsub#max_items">
                  <value>256</value>
                </field>
                <field type="list-single" var="pubsub#send_last_published_item">
                  <value>never</value>
                </field>
              </x>
            </publish-options>
          </pubsub>
        </iq>
        "#
    );

    recv!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="result">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="https://prose.org/protocol/room_settings">
              <item id="{{ROOM_ID}}" />
            </publish>
          </pubsub>
        </iq>
        "#
    );

    room.set_encryption_enabled(true).await;

    client.expect_load_device_list(&user_id!("user1@prose.org"), [100.into(), 101.into()]);
    client.expect_load_device_bundle(
        &user_id!("user1@prose.org"),
        &100.into(),
        Some(DeviceBundle::test(account_id!("user1@prose.org"), 100).await),
    );
    client.expect_load_device_bundle(
        &user_id!("user1@prose.org"),
        &101.into(),
        Some(DeviceBundle::test(account_id!("user1@prose.org"), 101).await),
    );

    client.expect_load_device_list(&user_id!("user2@prose.org"), [200.into()]);
    client.expect_load_device_bundle(
        &user_id!("user2@prose.org"),
        &200.into(),
        Some(DeviceBundle::test(account_id!("user2@prose.org"), 200).await),
    );

    let message_id = client.get_next_message_id();

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{MSG_ID}}" to="{{ROOM_ID}}" type="groupchat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="{{USER_DEVICE_ID}}">
              <key prekey="true" rid="100">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
              <key prekey="true" rid="101">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
              <key prekey="true" rid="200">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
              <iv>AQAAAAAAAAACAAAA</iv>
            </header>
            <payload>AgiX8ZA0voKAc/M=</payload>
          </encrypted>
          <encryption xmlns="urn:xmpp:eme:0" name="OMEMO" namespace="eu.siacs.conversations.axolotl" />
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    room_event!(
        client,
        room.jid().clone(),
        ClientRoomEventType::MessagesAppended {
            message_ids: vec![client.get_last_message_id()]
        }
    );

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello World".into(),
        }),
        attachments: vec![],
    })
    .await?;

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{OCCUPANT_ID}}" id="{{LAST_MSG_ID}}" to="{{USER_RESOURCE_ID}}" type="groupchat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="{{USER_DEVICE_ID}}">
              <key prekey="true" rid="100">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
              <key prekey="true" rid="101">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
              <key prekey="true" rid="200">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
              <iv>AQAAAAAAAAACAAAA</iv>
            </header>
            <payload>AgiX8ZA0voKAc/M=</payload>
          </encrypted>
          <encryption xmlns="urn:xmpp:eme:0" name="OMEMO" namespace="eu.siacs.conversations.axolotl" />
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="{{ANON_OCCUPANT_ID}}" />
          <stanza-id xmlns="urn:xmpp:sid:0" by="{{ROOM_ID}}" id="opZdWmO7r50ee_aGKnWvBMbK" />
        </message>
        "#
    );

    client.receive_next().await;

    let messages = room.load_messages_with_ids(&[message_id.into()]).await?;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages.first().unwrap().body.raw, "Hello World");

    Ok(())
}
