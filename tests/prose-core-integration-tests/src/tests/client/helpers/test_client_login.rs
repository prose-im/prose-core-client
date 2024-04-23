// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use crate::{recv, send};
use prose_core_client::dtos::UserId;
use prose_core_client::Secret;
use prose_xmpp::{ConnectionError, IDProvider};

use super::TestClient;

impl TestClient {
    pub async fn perform_login(
        &self,
        user: UserId,
        password: impl AsRef<str>,
    ) -> Result<(), ConnectionError> {
        self.push_ctx(
            [
                ("USER_ID".into(), user.to_string()),
                (
                    "USER_RESOURCE_ID".into(),
                    format!("{}/{}", user.to_string(), self.short_id_provider.new_id()),
                ),
            ]
            .into(),
        );

        self.perform_load_roster();

        // Initial presence
        send!(
            self,
            r#"
        <presence xmlns='jabber:client'>
            <show>chat</show>
            <c xmlns='http://jabber.org/protocol/caps' hash="sha-1" node="https://prose.org" ver="6F3DapJergay3XYdZEtLkCjrPpc=" />
        </presence>"#
        );

        // Enable carbons
        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" type="set">
            <enable xmlns='urn:xmpp:carbons:2'/>
        </iq>"#
        );
        recv!(
            self,
            r#"<iq xmlns="jabber:client" id="{{ID}}" type="result" />"#
        );

        self.perform_request_server_capabilities();
        self.perform_load_block_list();
        self.perform_load_device_list();
        self.perform_publish_device();
        self.perform_publish_device_bundle();

        self.connect(&user, Secret::new(password.as_ref().to_string()))
            .await
    }
}

impl TestClient {
    fn perform_load_roster(&self) {
        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" type="get">
            <query xmlns='jabber:iq:roster'/>
        </iq>
        "#
        );
        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" type="result">
            <query xmlns="jabber:iq:roster" ver="1" />
        </iq>
        "#
        );
    }

    fn perform_request_server_capabilities(&self) {
        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="prose.org" type="get">
            <query xmlns='http://jabber.org/protocol/disco#items'/>
        </iq>"#
        );
        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" from="nsm.chat" id="{{ID}}" type="result">
            <query xmlns="http://jabber.org/protocol/disco#items">
                <item jid="conference.prose.org" name="Chatrooms" />
                <item jid="upload.prose.org" name="HTTP File Upload" />
            </query>
        </iq>
        "#
        );

        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="conference.prose.org" type="get">
            <query xmlns='http://jabber.org/protocol/disco#info'/>
        </iq>"#
        );
        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" from="conference.nsm.chat" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/disco#info">
            <identity category="conference" name="Chatrooms" type="text" />
            <feature var="http://jabber.org/protocol/muc#unique" />
            <feature var="urn:xmpp:mam:2" />
            <feature var="http://jabber.org/protocol/commands" />
            <feature var="http://jabber.org/protocol/disco#info" />
            <feature var="http://jabber.org/protocol/disco#items" />
            <feature var="urn:xmpp:occupant-id:0" />
            <feature var="http://jabber.org/protocol/muc" />
            <x xmlns="jabber:x:data" type="result">
              <field type="hidden" var="FORM_TYPE">
                <value>http://jabber.org/network/serverinfo</value>
              </field>
              <field type="list-multi" var="abuse-addresses" />
              <field type="list-multi" var="admin-addresses">
                <value>mailto:hostmaster@prose.org.local</value>
              </field>
              <field type="list-multi" var="feedback-addresses" />
              <field type="list-multi" var="sales-addresses" />
              <field type="list-multi" var="security-addresses" />
              <field type="list-multi" var="status-addresses" />
              <field type="list-multi" var="support-addresses" />
            </x>
          </query>
        </iq>
        "#
        );

        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="upload.prose.org" type="get">
            <query xmlns='http://jabber.org/protocol/disco#info'/>
        </iq>"#
        );
        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" from="upload.nsm.chat" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/disco#info">
            <identity category="store" name="HTTP File Upload" type="file" />
            <feature var="urn:xmpp:http:upload:0" />
            <feature var="http://jabber.org/protocol/disco#info" />
            <feature var="http://jabber.org/protocol/disco#items" />
            <x xmlns="jabber:x:data" type="result">
              <field type="hidden" var="FORM_TYPE">
                <value>http://jabber.org/network/serverinfo</value>
              </field>
              <field type="list-multi" var="abuse-addresses" />
              <field type="list-multi" var="admin-addresses">
                <value>mailto:hostmaster@prose.org.local</value>
              </field>
              <field type="list-multi" var="feedback-addresses" />
              <field type="list-multi" var="sales-addresses" />
              <field type="list-multi" var="security-addresses" />
              <field type="list-multi" var="status-addresses" />
              <field type="list-multi" var="support-addresses" />
            </x>
            <x xmlns="jabber:x:data" type="result">
              <field type="hidden" var="FORM_TYPE">
                <value>urn:xmpp:http:upload:0</value>
              </field>
              <field type="text-single" var="max-file-size">
                <value>16777216</value>
              </field>
            </x>
          </query>
        </iq>
        "#
        );
    }

    fn perform_load_block_list(&self) {
        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" type="get">
            <blocklist xmlns='urn:xmpp:blocking'/>
        </iq>"#
        );
        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" type="result">
          <blocklist xmlns="urn:xmpp:blocking" />
        </iq>
        "#
        );
    }

    fn perform_publish_device(&self) {
        send!(
            self,
            r#"
            <iq xmlns='jabber:client' id="{{ID}}" type="set">
              <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                <publish node="eu.siacs.conversations.axolotl.devicelist">
                  <item id="current">
                    <list xmlns='eu.siacs.conversations.axolotl'>
                      <device id="{{USER_DEVICE_ID}}" label="prose-core-client" />
                    </list>
                  </item>
                </publish><publish-options>
                  <x xmlns='jabber:x:data' type="submit">
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
            self,
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
    }
}
