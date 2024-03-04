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
        self.perform_load_roster();

        // Initial presence
        self.send(r#"
        <presence xmlns='jabber:client'>
            <show>chat</show>
            <c xmlns='http://jabber.org/protocol/caps' hash="sha-1" node="https://prose.org" ver="dWkYhmO8yFRrk+2G6R324ES7G9E=" />
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
}
