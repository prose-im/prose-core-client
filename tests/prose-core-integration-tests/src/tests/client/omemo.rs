// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::dtos::{SendMessageRequest, SendMessageRequestBody, UserId};
use prose_core_client::user_id;
use prose_proc_macros::mt_test;

use crate::{recv, send};

use super::helpers::TestClient;

#[mt_test]
async fn test_receives_device_list_with_current_device_missing() -> Result<()> {
    let client = TestClient::new().await;

    client
        .perform_login(user_id!("user@prose.org"), "secret")
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
        .perform_login(user_id!("user@prose.org"), "secret")
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
        .perform_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room = client
        .perform_start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>Hello World</body>
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
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
        .perform_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room = client
        .perform_start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    room.set_encryption_enabled(true);

    client.perform_start_omemo_session(user_id!("them@prose.org"), []);

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
        .perform_login(user_id!("user@prose.org"), "secret")
        .await?;

    let room = client
        .perform_start_dm(user_id!("them@prose.org"))
        .await?
        .to_generic_room();

    room.set_encryption_enabled(true);

    client.perform_start_omemo_session(user_id!("them@prose.org"), [111.into(), 222.into()]);

    send!(
        client,
        r#"
        <message xmlns="jabber:client" from="{{USER_RESOURCE_ID}}" id="{{ID}}" to="them@prose.org" type="chat">
          <body>[This message is OMEMO encrypted]</body>
          <encrypted xmlns="eu.siacs.conversations.axolotl">
            <header sid="12345">
              <key prekey="true" rid="111">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwrF71OY7D2TlW90xkGelF7p4fWb9vdPcryLn4tH8v72CiI5FGhRkoGz/r5ZkbQ02gHDLjGDJNp3wouWAwAA==</key>
              <key prekey="true" rid="222">NAgBEiEFND1CvWJkfRtiPCbfg05oqhzeDZsIozOyrJ42KE2BpTEaIQU0PUK9YmR9G2I8Jt+DTmiqHN4NmwijM7KsnjYoTYGlMSJiNAohBTy2vpu/tngnkcXIepkXEexslHOd/zcO5NssELTdoyJuEAAYACIwrF71OY7D2TlW90xkGelF7p4fWb9vdPcryLn4tH8v72CiI5FGhRkoGz/r5ZkbQ02gHDLjGDJNp3wouWAwAA==</key>
              <iv>AQAAAAAAAAACAAAA</iv>
            </header>
            <payload>AgiX8ZA0voKAc/M=</payload>
          </encrypted>
          <encryption xmlns="urn:xmpp:eme:0" name="OMEMO" namespace="eu.siacs.conversations.axolotl" />
          <active xmlns="http://jabber.org/protocol/chatstates" />
          <markable xmlns="urn:xmpp:chat-markers:0" />
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
