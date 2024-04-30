// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use minidom::Element;

use prose_core_client::domain::rooms::services::impls::build_nickname;
use prose_core_client::domain::sidebar::models::BookmarkType;
use prose_core_client::dtos::{
    DeviceBundle, DeviceId, DeviceInfo, DeviceTrust, MucId, RoomId, SendMessageRequest,
    SendMessageRequestBody, UserId,
};
use prose_core_client::{muc_id, user_id};
use prose_proc_macros::mt_test;

use crate::{recv, send};

use super::helpers::{LoginConfig, TestClient, TestDeviceBundle};

#[mt_test]
async fn test_receives_device_list_with_current_device_missing() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    recv!(
        client,
        r#"
      <message xmlns="jabber:client" from="{{USER_ID}}" id="some-id" to="{{USER_ID}}" type="headline">
        <event xmlns="http://jabber.org/protocol/pubsub#event">
          <items node="eu.siacs.conversations.axolotl.devicelist">
            <item id="current" publisher="{{USER_ID}}">
              <list xmlns="eu.siacs.conversations.axolotl">
                <device id="1" />
                <device id="2" />
                <device id="3" />
              </list>
            </item>
          </items>
        </event>
      </message>
      "#
    );

    send!(
        client,
        r#"
        <iq xmlns="jabber:client" id="{{ID}}" type="set">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="eu.siacs.conversations.axolotl.devicelist">
              <item id="current">
                <list xmlns="eu.siacs.conversations.axolotl">
                  <device id="1" />
                  <device id="2" />
                  <device id="3" />
                  <device id="{{USER_DEVICE_ID}}" label="prose-core-client" />
                </list>
              </item>
            </publish>
            <publish-options>
              <x xmlns="jabber:x:data" type="submit">
                <field type="hidden" var="FORM_TYPE">
                  <value>http://jabber.org/protocol/pubsub#publish-options</value>
                </field>
                <field var="pubsub#access_model">
                  <value>open</value>
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
            <publish node="eu.siacs.conversations.axolotl.devicelist">
              <item id="current" />
            </publish>
          </pubsub>
        </iq>
        "#
    );

    client.receive_next().await;

    Ok(())
}

#[mt_test]
async fn test_receives_device_list_with_current_device_included() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    recv!(
        client,
        r#"
      <message xmlns="jabber:client" from="{{USER_ID}}" id="some-id" to="{{USER_ID}}" type="headline">
        <event xmlns="http://jabber.org/protocol/pubsub#event">
          <items node="eu.siacs.conversations.axolotl.devicelist">
            <item id="current" publisher="{{USER_ID}}">
              <list xmlns="eu.siacs.conversations.axolotl">
                <device id="1" />
                <device id="2" />
                <device id="3" />
                <device id="{{USER_DEVICE_ID}}" />
              </list>
            </item>
          </items>
        </event>
      </message>
      "#
    );

    client.receive_next().await;

    Ok(())
}

#[mt_test]
async fn test_does_not_start_session_when_sending_message_in_non_encrypted_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room = client
        .start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>Hello World</body>
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello World".to_string(),
            mentions: vec![],
        }),
        attachments: vec![],
    })
    .await?;

    Ok(())
}

#[mt_test]
async fn test_sending_encrypted_message_fails_if_recipient_has_no_devices() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room = client
        .start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    room.set_encryption_enabled(true);

    client.expect_load_device_list(&user_id!("them@prose.org"), []);

    let result = room
        .send_message(SendMessageRequest {
            body: Some(SendMessageRequestBody {
                text: "Hello World".to_string(),
                mentions: vec![],
            }),
            attachments: vec![],
        })
        .await;

    // TODO: Check for correct error
    assert!(result.is_err());

    Ok(())
}

#[mt_test]
async fn test_start_session_when_sending_message_in_encrypted_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login_with_config(
            user_id!("user@prose.org"),
            "secret",
            LoginConfig::default()
                .with_device_bundles([(500.into(), DeviceBundle::test(500).await)]),
        )
        .await?;

    let room = client
        .start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    room.set_encryption_enabled(true);

    // Device list is not loaded here, because it is already cached.
    client.expect_load_device_bundle(
        &user_id!("user@prose.org"),
        &500.into(),
        Some(DeviceBundle::test(500).await),
    );

    client.expect_load_device_list(&user_id!("them@prose.org"), [111.into(), 222.into()]);
    client.expect_load_device_bundle(
        &user_id!("them@prose.org"),
        &111.into(),
        Some(DeviceBundle::test(111).await),
    );
    client.expect_load_device_bundle(
        &user_id!("them@prose.org"),
        &222.into(),
        Some(DeviceBundle::test(222).await),
    );

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="{{USER_DEVICE_ID}}">
              <key prekey="true" rid="500">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
              <key prekey="true" rid="111">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
              <key prekey="true" rid="222">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
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

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello World".to_string(),
            mentions: vec![],
        }),
        attachments: vec![],
    })
    .await?;

    // Sessions should only be started once…

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="{{USER_DEVICE_ID}}">
              <key prekey="true" rid="500">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAEYACIwl6iMdEngbUMSj+lMfNEg4dgEruhF+Jnlj81us5vNX6WjXZulX3+kAmUi3JuRfjs3lw4pxCbZop0ouWAwAA==</key>
              <key prekey="true" rid="111">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAEYACIwl6iMdEngbUMSj+lMfNEg4dgEruhF+Jnlj81us5vNX6WjXZulX3+kAmUi3JuRfjs3lw4pxCbZop0ouWAwAA==</key>
              <key prekey="true" rid="222">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAEYACIwl6iMdEngbUMSj+lMfNEg4dgEruhF+Jnlj81us5vNX6WjXZulX3+kAmUi3JuRfjs3lw4pxCbZop0ouWAwAA==</key>
              <iv>AQAAAAAAAAACAAAA</iv>
            </header>
            <payload>AgiX8ZA0voKAc/MDFA==</payload>
          </encrypted>
          <encryption xmlns="urn:xmpp:eme:0" name="OMEMO" namespace="eu.siacs.conversations.axolotl" />
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello World 2".to_string(),
            mentions: vec![],
        }),
        attachments: vec![],
    })
    .await?;

    Ok(())
}

#[mt_test]
async fn test_starts_session_for_new_devices_when_sending() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room = client
        .start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    room.set_encryption_enabled(true);

    client.expect_load_device_list(&user_id!("them@prose.org"), [111.into()]);
    client.expect_load_device_bundle(
        &user_id!("them@prose.org"),
        &111.into(),
        Some(DeviceBundle::test(111).await),
    );

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="{{USER_DEVICE_ID}}">
              <key prekey="true" rid="111">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
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

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello World".to_string(),
            mentions: vec![],
        }),
        attachments: vec![],
    })
    .await?;

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="them@prose.org" id="some-id" to="{{USER_RESOURCE_ID}}" type="headline">
          <event xmlns="http://jabber.org/protocol/pubsub#event">
            <items node="eu.siacs.conversations.axolotl.devicelist">
              <item id="current" publisher="them@prose.org">
                <list xmlns="eu.siacs.conversations.axolotl">
                  <device id="111" />
                  <device id="222" />
                </list>
              </item>
            </items>
          </event>
        </message>
        "#
    );

    client.receive_next().await;

    client.expect_load_device_bundle(
        &user_id!("them@prose.org"),
        &222.into(),
        Some(DeviceBundle::test(222).await),
    );

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="{{USER_DEVICE_ID}}">
              <key prekey="true" rid="111">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAEYACIwl6iMdEngbUMSj+lMfNEg4dgEruhF+Jnlj81us5vNX6WjXZulX3+kAmUi3JuRfjs3lw4pxCbZop0ouWAwAA==</key>
              <key prekey="true" rid="222">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1Sf4/ncqWJNWEKyKT2hw4oH8PJ/AQllR1jun4CNQyBBJgCX29EC2+DSIouWAwAA==</key>
              <iv>AQAAAAAAAAACAAAA</iv>
            </header>
            <payload>AgiX8ZA0voKAc/MDFA==</payload>
          </encrypted>
          <encryption xmlns="urn:xmpp:eme:0" name="OMEMO" namespace="eu.siacs.conversations.axolotl" />
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello World 2".to_string(),
            mentions: vec![],
        }),
        attachments: vec![],
    })
    .await?;

    Ok(())
}

#[mt_test]
async fn test_marks_disappeared_devices_as_inactive_and_reappeared_as_active() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room = client
        .start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    room.set_encryption_enabled(true);

    client.expect_load_device_list(&user_id!("them@prose.org"), [111.into(), 222.into()]);
    client.expect_load_device_bundle(
        &user_id!("them@prose.org"),
        &111.into(),
        Some(DeviceBundle::test(111).await),
    );
    client.expect_load_device_bundle(
        &user_id!("them@prose.org"),
        &222.into(),
        Some(DeviceBundle::test(222).await),
    );

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="{{USER_DEVICE_ID}}">
              <key prekey="true" rid="111">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
              <key prekey="true" rid="222">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
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

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello World".to_string(),
            mentions: vec![],
        }),
        attachments: vec![],
    })
    .await?;

    let device_infos = client
        .user_data
        .load_user_device_infos(&user_id!("them@prose.org"))
        .await?;

    assert_eq!(
        vec![
            DeviceInfoTest::new(111, DeviceTrust::Undecided, true, false),
            DeviceInfoTest::new(222, DeviceTrust::Undecided, true, false)
        ],
        device_infos.into_device_info_test(),
    );

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="them@prose.org" id="some-id" to="{{USER_RESOURCE_ID}}" type="headline">
          <event xmlns="http://jabber.org/protocol/pubsub#event">
            <items node="eu.siacs.conversations.axolotl.devicelist">
              <item id="current" publisher="them@prose.org">
                <list xmlns="eu.siacs.conversations.axolotl">
                  <!-- Device with id 111 disappeared -->
                  <device id="222" />
                  <!-- Device with id 333 appeared -->
                  <device id="333" />
                </list>
              </item>
            </items>
          </event>
        </message>
        "#
    );

    client.receive_next().await;

    client.expect_load_device_bundle(
        &user_id!("them@prose.org"),
        &333.into(),
        Some(DeviceBundle::test(333).await),
    );

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="{{USER_DEVICE_ID}}">
              <key prekey="true" rid="222">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAEYACIwl6iMdEngbUMSj+lMfNEg4WPisea0uLclQu/qX56CBUjneijwMw+0GarLuvCCtS4Ajh2ChS2B48souWAwAA==</key>
              <key prekey="true" rid="333">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
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

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello World".to_string(),
            mentions: vec![],
        }),
        attachments: vec![],
    })
    .await?;

    let device_infos = client
        .user_data
        .load_user_device_infos(&user_id!("them@prose.org"))
        .await?;

    assert_eq!(
        vec![
            DeviceInfoTest::new(111, DeviceTrust::Undecided, false, false),
            DeviceInfoTest::new(222, DeviceTrust::Undecided, true, false),
            DeviceInfoTest::new(333, DeviceTrust::Undecided, true, false),
        ],
        device_infos.into_device_info_test(),
    );

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="them@prose.org" id="some-id" to="{{USER_RESOURCE_ID}}" type="headline">
          <event xmlns="http://jabber.org/protocol/pubsub#event">
            <items node="eu.siacs.conversations.axolotl.devicelist">
              <item id="current" publisher="them@prose.org">
                <list xmlns="eu.siacs.conversations.axolotl">
                  <!-- Device with id 111 reappeared -->
                  <device id="111" />
                  <device id="222" />
                  <device id="333" />
                </list>
              </item>
            </items>
          </event>
        </message>
        "#
    );

    client.receive_next().await;

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="{{USER_DEVICE_ID}}">
              <key prekey="true" rid="111">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAEYACIwl6iMdEngbUMSj+lMfNEg4WPisea0uLclQu/qX56CBUjneijwMw+0GarLuvCCtS4Ajh2ChS2B48souWAwAA==</key>
              <key prekey="true" rid="222">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAIYACIwhSg5zKWyLQDkFe323V24vIr52xSFaekVRtv3kNoCuOVec2dQBTGNHq+0gQPBmnzMho38m95uf0IouWAwAA==</key>
              <key prekey="true" rid="333">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAEYACIwl6iMdEngbUMSj+lMfNEg4WPisea0uLclQu/qX56CBUjneijwMw+0GarLuvCCtS4Ajh2ChS2B48souWAwAA==</key>
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

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello World".to_string(),
            mentions: vec![],
        }),
        attachments: vec![],
    })
    .await?;

    Ok(())
}

#[mt_test]
async fn test_marks_own_disappeared_devices_as_inactive() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login_with_config(
            user_id!("user@prose.org"),
            "secret",
            LoginConfig::default().with_device_bundles([
                (10.into(), DeviceBundle::test(10).await),
                (20.into(), DeviceBundle::test(20).await),
            ]),
        )
        .await?;

    let room = client
        .start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    room.set_encryption_enabled(true);

    client.expect_load_device_bundle(
        &user_id!("user@prose.org"),
        &10.into(),
        Some(DeviceBundle::test(10).await),
    );
    client.expect_load_device_bundle(
        &user_id!("user@prose.org"),
        &20.into(),
        Some(DeviceBundle::test(20).await),
    );

    client.expect_load_device_list(&user_id!("them@prose.org"), [100.into()]);
    client.expect_load_device_bundle(
        &user_id!("them@prose.org"),
        &100.into(),
        Some(DeviceBundle::test(100).await),
    );

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="{{USER_DEVICE_ID}}">
              <key prekey="true" rid="10">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
              <key prekey="true" rid="20">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
              <key prekey="true" rid="100">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwaeNDn7QTbNfd3kVdI6q1SSL78yHI2fohvZq3XHM+xzGDLjJQGDsaOlkA3gFblk3imzOLObPtCmkouWAwAA==</key>
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

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello World".to_string(),
            mentions: vec![],
        }),
        attachments: vec![],
    })
    .await?;

    recv!(
        client,
        r#"
      <message xmlns="jabber:client" from="{{USER_ID}}" id="some-id" to="{{USER_ID}}" type="headline">
        <event xmlns="http://jabber.org/protocol/pubsub#event">
          <items node="eu.siacs.conversations.axolotl.devicelist">
            <item id="current" publisher="{{USER_ID}}">
              <list xmlns="eu.siacs.conversations.axolotl">
                <device id="20" />
                <device id="{{USER_DEVICE_ID}}" />
              </list>
            </item>
          </items>
        </event>
      </message>
      "#
    );

    client.receive_next().await;

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="{{USER_DEVICE_ID}}">
              <key prekey="true" rid="20">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAEYACIwl6iMdEngbUMSj+lMfNEg4WPisea0uLclQu/qX56CBUjneijwMw+0GarLuvCCtS4Ajh2ChS2B48souWAwAA==</key>
              <key prekey="true" rid="100">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAEYACIwl6iMdEngbUMSj+lMfNEg4WPisea0uLclQu/qX56CBUjneijwMw+0GarLuvCCtS4Ajh2ChS2B48souWAwAA==</key>
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

    room.send_message(SendMessageRequest {
        body: Some(SendMessageRequestBody {
            text: "Hello World".to_string(),
            mentions: vec![],
        }),
        attachments: vec![],
    })
    .await?;

    Ok(())
}

#[mt_test]
async fn test_starts_session_and_decrypts_received_messages() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room = client
        .start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    assert!(client
        .user_data
        .load_user_device_infos(&user_id!("them@prose.org"))
        .await?
        .is_empty());

    let service = TestClient::their_encryption_domain_service(user_id!("them@prose.org")).await;
    let encrypted_payload = service
        .encrypt_message(
            &user_id!("user@prose.org"),
            "Can you read this?".to_string(),
        )
        .await?;

    client.push_ctx(
        [(
            "ENCRYPTED_PAYLOAD".into(),
            String::from(&Element::from(xmpp_parsers::legacy_omemo::Encrypted::from(
                encrypted_payload,
            ))),
        )]
        .into(),
    );

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="them@prose.org" id="my-message-id" to="{{USER_ID}}" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          {{ENCRYPTED_PAYLOAD}}
          <encryption xmlns="urn:xmpp:eme:0" name="OMEMO" namespace="eu.siacs.conversations.axolotl" />
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
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
        <message xmlns="jabber:client" from="user@prose.org/short-id-1" to="them@prose.org" type="chat">
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

    client.receive_next().await;

    let messages = room
        .load_messages_with_ids(&["my-message-id".into()])
        .await?;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages.first().unwrap().body, "Can you read this?");

    let device_infos = client
        .user_data
        .load_user_device_infos(&user_id!("them@prose.org"))
        .await?;

    assert_eq!(
        vec![DeviceInfoTest::new(
            TestClient::their_device_id(),
            DeviceTrust::Undecided,
            true,
            false
        )],
        device_infos.into_device_info_test(),
    );

    let encrypted_payload = service
        .encrypt_message(
            &user_id!("user@prose.org"),
            "Can you read this too?".to_string(),
        )
        .await?;

    client.push_ctx(
        [(
            "ENCRYPTED_PAYLOAD".into(),
            String::from(&Element::from(xmpp_parsers::legacy_omemo::Encrypted::from(
                encrypted_payload,
            ))),
        )]
        .into(),
    );

    recv!(
        client,
        r#"
        <message xmlns="jabber:client" from="them@prose.org" id="other-message-id" to="{{USER_ID}}" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          {{ENCRYPTED_PAYLOAD}}
          <encryption xmlns="urn:xmpp:eme:0" name="OMEMO" namespace="eu.siacs.conversations.axolotl" />
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="user@prose.org/short-id-1" to="them@prose.org" type="chat">
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="12345">
              <key rid="54321">NAohBTQ9Qr1iZH0bYjwm34NOaKoc3g2bCKMzsqyeNihNgaUxEAEYACIwUu7QgsZvpXMb0riDHio8fWTCbiyCDLRswhbCpYly4bhed9ZgyYuenxIq7kE+q4g0nWh4Pjd1QSQ=</key>
              <iv>AQAAAAAAAAACAAAA</iv>
            </header>
          </encrypted>
          <encryption xmlns="urn:xmpp:eme:0" name="OMEMO" namespace="eu.siacs.conversations.axolotl" />
          <store xmlns="urn:xmpp:hints" />
        </message>
        "#
    );

    client.receive_next().await;

    // Second message should not contain a pre-key, thus the bundle shouldn't be published again.

    let messages = room
        .load_messages_with_ids(&["other-message-id".into()])
        .await?;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages.first().unwrap().body, "Can you read this too?");

    Ok(())
}

#[mt_test]
async fn test_decrypts_message_from_private_nonanonymous_muc_room() -> Result<()> {
    let client = TestClient::new().await;

    client
        .expect_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room_id = muc_id!("room@prose.org");
    let room_name = "omemo-private-channel";
    let nickname = build_nickname(
        &client
            .connected_user_id()
            .expect("You're not connected")
            .into_user_id(),
    );
    let occupant_id = room_id.occupant_id_with_nickname(nickname)?;

    client.push_ctx(
        [
            ("OCCUPANT_ID".into(), occupant_id.to_string()),
            ("ANON_OCCUPANT_ID".into(), "some-weird-id".to_string()),
            ("ROOM_ID".into(), room_id.to_string()),
            ("ROOM_NAME".into(), room_name.to_string()),
        ]
        .into(),
    );

    send!(
        client,
        r#"
        <presence xmlns='jabber:client' to="{{OCCUPANT_ID}}">
          <show>chat</show>
          <x xmlns='http://jabber.org/protocol/muc'>
            <history maxstanzas="0" />
          </x>
          <c xmlns='http://jabber.org/protocol/caps' hash="sha-1" node="https://prose.org" ver="6F3DapJergay3XYdZEtLkCjrPpc="/>
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
          <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="https://prose.org" ver="6F3DapJergay3XYdZEtLkCjrPpc=" />
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

    client.expect_send_vard_request(&user_id!("user1@prose.org"));
    client.receive_not_found_iq_response();

    client.expect_send_vard_request(&user_id!("user2@prose.org"));
    client.receive_not_found_iq_response();

    client.expect_send_vard_request(&user_id!("user3@prose.org"));
    client.receive_not_found_iq_response();

    client.expect_set_bookmark(
        &RoomId::Muc(room_id.clone()),
        room_name,
        BookmarkType::PrivateChannel,
    );

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
            &user_id!("user@prose.org"),
            "Can you read this?".to_string(),
        )
        .await?;

    client.push_ctx(
        [(
            "ENCRYPTED_PAYLOAD".into(),
            String::from(&Element::from(xmpp_parsers::legacy_omemo::Encrypted::from(
                encrypted_payload,
            ))),
        )]
        .into(),
    );

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

    client.receive_next().await;

    let messages = room
        .load_messages_with_ids(&["my-message-id".into()])
        .await?;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages.first().unwrap().body, "Can you read this?");

    Ok(())
}

#[derive(Debug, PartialEq)]
struct DeviceInfoTest {
    pub id: DeviceId,
    pub trust: DeviceTrust,
    pub is_active: bool,
    pub is_this_device: bool,
}

impl DeviceInfoTest {
    pub fn new(
        id: impl Into<DeviceId>,
        trust: DeviceTrust,
        is_active: bool,
        is_this_device: bool,
    ) -> Self {
        Self {
            id: id.into(),
            trust,
            is_active,
            is_this_device,
        }
    }
}

impl From<DeviceInfo> for DeviceInfoTest {
    fn from(value: DeviceInfo) -> Self {
        Self {
            id: value.id,
            trust: value.trust,
            is_active: value.is_active,
            is_this_device: value.is_this_device,
        }
    }
}

trait IntoDeviceInfoTest {
    fn into_device_info_test(self) -> Vec<DeviceInfoTest>;
}

impl IntoDeviceInfoTest for Vec<DeviceInfo> {
    fn into_device_info_test(self) -> Vec<DeviceInfoTest> {
        self.into_iter().map(DeviceInfoTest::from).collect()
    }
}
