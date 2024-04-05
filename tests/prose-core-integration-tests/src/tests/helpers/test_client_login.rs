// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::dtos::UserId;
use prose_xmpp::ConnectionError;

use super::TestClient;

impl TestClient {
    pub async fn perform_login(
        &self,
        user: UserId,
        password: impl AsRef<str>,
    ) -> Result<(), ConnectionError> {
        self.push_ctx([("USER_ID".into(), user.to_string())].into());

        self.perform_load_roster();

        // Initial presence
        self.send(r#"
        <presence xmlns='jabber:client'>
            <show>chat</show>
            <c xmlns='http://jabber.org/protocol/caps' hash="sha-1" node="https://prose.org" ver="6F3DapJergay3XYdZEtLkCjrPpc=" />
        </presence>"#
        );

        // Enable carbons
        self.send(
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" type="set">
            <enable xmlns='urn:xmpp:carbons:2'/>
        </iq>"#,
        );
        self.receive(r#"<iq xmlns="jabber:client" id="{{ID}}" type="result" />"#);

        self.perform_request_server_capabilities();
        self.perform_load_block_list();
        self.perform_load_device_list();
        self.perform_publish_device();
        self.perform_publish_device_bundle();
        self.perform_start_session(&user);

        self.connect(&user, password).await
    }
}

impl TestClient {
    fn perform_load_roster(&self) {
        self.send(
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" type="get">
            <query xmlns='jabber:iq:roster'/>
        </iq>
        "#,
        );
        self.receive(
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" type="result">
            <query xmlns="jabber:iq:roster" ver="1" />
        </iq>
        "#,
        );
    }

    fn perform_request_server_capabilities(&self) {
        self.send(
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="prose.org" type="get">
            <query xmlns='http://jabber.org/protocol/disco#items'/>
        </iq>"#,
        );
        self.receive(
            r#"
        <iq xmlns="jabber:client" from="nsm.chat" id="{{ID}}" type="result">
            <query xmlns="http://jabber.org/protocol/disco#items">
                <item jid="conference.prose.org" name="Chatrooms" />
                <item jid="upload.prose.org" name="HTTP File Upload" />
            </query>
        </iq>
        "#,
        );

        self.send(
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="conference.prose.org" type="get">
            <query xmlns='http://jabber.org/protocol/disco#info'/>
        </iq>"#,
        );
        self.receive(
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
        "#,
        );

        self.send(
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="upload.prose.org" type="get">
            <query xmlns='http://jabber.org/protocol/disco#info'/>
        </iq>"#,
        );
        self.receive(
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
        "#,
        );
    }

    fn perform_load_block_list(&self) {
        self.send(
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" type="get">
            <blocklist xmlns='urn:xmpp:blocking'/>
        </iq>"#,
        );
        self.receive(
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" type="result">
          <blocklist xmlns="urn:xmpp:blocking" />
        </iq>
        "#,
        );
    }

    fn perform_load_device_list(&self) {
        self.send(
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="{{USER_ID}}" type="get">
            <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                <items node="eu.siacs.conversations.axolotl.devicelist"/>
            </pubsub>
        </iq>
        "#,
        );
        self.receive(
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="result">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <items node="eu.siacs.conversations.axolotl.devicelist">
              <item id="current">
                <list xmlns="eu.siacs.conversations.axolotl" />
              </item>
            </items>
          </pubsub>
        </iq>
            "#,
        )
    }

    fn perform_publish_device(&self) {
        self.send(
            r#"
            <iq xmlns='jabber:client' id="{{ID}}" type="set">
              <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                <publish node="eu.siacs.conversations.axolotl.devicelist">
                  <item id="current">
                    <list xmlns='eu.siacs.conversations.axolotl'>
                      <device id="0" label="prose-core-client" />
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
            "#,
        );
        self.receive(
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="result">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="eu.siacs.conversations.axolotl.devicelist">
              <item id="current" />
            </publish>
          </pubsub>
        </iq>
        "#,
        );
    }

    fn perform_publish_device_bundle(&self) {
        self.send(
            r#"
            <iq xmlns='jabber:client' id="{{ID}}" to="{{USER_ID}}" type="get">
              <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                <items node="eu.siacs.conversations.axolotl.bundles:0" />
              </pubsub>
            </iq>
            "#,
        );
        self.receive(
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="error">
              <error type="cancel">
                <item-not-found xmlns="urn:ietf:params:xml:ns:xmpp-stanzas" />
              </error>
            </iq>
            "#,
        );

        self.send(
            r#"
            <iq xmlns='jabber:client' id="{{ID}}" type="set">
              <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                <publish node="eu.siacs.conversations.axolotl.bundles:0">
                  <item id="current">
                    <bundle xmlns='eu.siacs.conversations.axolotl'>
                      <signedPreKeyPublic signedPreKeyId="0">AA==</signedPreKeyPublic>
                      <signedPreKeySignature>AA==</signedPreKeySignature>
                      <identityKey>AA==</identityKey>
                      <prekeys />
                    </bundle>
                  </item>
                </publish>
                <publish-options>
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
            "#,
        );
        self.receive(
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="result">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="eu.siacs.conversations.axolotl.bundles:0">
              <item id="current" />
            </publish>
          </pubsub>
        </iq>
        "#,
        );
    }

    fn perform_start_session(&self, user_id: &UserId) {
        self.push_ctx([("OTHER_USER_ID".into(), user_id.to_string())].into());

        self.send(
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="{{OTHER_USER_ID}}" type="get">
            <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                <items node="eu.siacs.conversations.axolotl.devicelist"/>
            </pubsub>
        </iq>
        "#,
        );
        self.receive(
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{OTHER_USER_ID}}" type="result">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <items node="eu.siacs.conversations.axolotl.devicelist">
              <item id="current">
                <list xmlns="eu.siacs.conversations.axolotl" />
              </item>
            </items>
          </pubsub>
        </iq>
            "#,
        );

        self.pop_ctx();
    }
}
