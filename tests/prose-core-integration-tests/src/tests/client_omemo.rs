// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::dtos::UserId;
use prose_core_client::user_id;
use prose_proc_macros::mt_test;

use crate::tests::helpers::TestClient;
use crate::{recv, send};

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
