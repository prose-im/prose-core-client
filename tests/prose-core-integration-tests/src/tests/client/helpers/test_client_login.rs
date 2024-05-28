// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::dtos::{DeviceBundle, DeviceId, UserId};
use prose_core_client::{ClientEvent, ConnectionEvent, Secret};
use prose_xmpp::{ConnectionError, IDProvider};

use crate::{event, recv, send};

use super::TestClient;

#[derive(Default)]
pub struct LoginConfig {
    pub device_bundles: Vec<(DeviceId, DeviceBundle)>,
}

impl LoginConfig {
    pub fn with_device_bundles(
        mut self,
        device_bundles: impl IntoIterator<Item = (DeviceId, DeviceBundle)>,
    ) -> Self {
        self.device_bundles = device_bundles.into_iter().collect::<Vec<_>>();
        self
    }
}

impl TestClient {
    pub async fn expect_login(
        &self,
        user: UserId,
        password: impl AsRef<str>,
    ) -> Result<(), ConnectionError> {
        self.expect_login_with_config(user, password, LoginConfig::default())
            .await
    }

    pub async fn expect_login_with_config(
        &self,
        user: UserId,
        password: impl AsRef<str>,
        config: LoginConfig,
    ) -> Result<(), ConnectionError> {
        self.push_ctx(
            [
                ("USER_ID".into(), user.to_string()),
                (
                    "USER_RESOURCE_ID".into(),
                    format!("{}/{}", user.to_string(), self.short_id_provider.new_id()),
                ),
                (
                    "CAPS_HASH".into(),
                    "zQudvh/0QdfUMrrQBB1ZR3NMyTY=".to_string(),
                ),
            ]
            .into(),
        );

        self.expect_load_roster();

        // Initial presence
        send!(
            self,
            r#"
        <presence xmlns='jabber:client'>
            <show>chat</show>
            <c xmlns='http://jabber.org/protocol/caps' hash="sha-1" node="https://prose.org" ver="{{CAPS_HASH}}" />
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

        self.expect_request_server_capabilities();
        self.expect_load_block_list();

        self.expect_load_device_list(
            &user,
            config.device_bundles.clone().into_iter().map(|(id, _)| id),
        );
        self.expect_publish_device(config.device_bundles.into_iter().map(|(id, _)| id));
        self.expect_publish_initial_device_bundle();

        event!(
            self,
            ClientEvent::ConnectionStatusChanged {
                event: ConnectionEvent::Connect,
            }
        );

        event!(self, ClientEvent::AccountInfoChanged);

        self.connect(&user, Secret::new(password.as_ref().to_string()))
            .await
    }
}

impl TestClient {
    fn expect_load_roster(&self) {
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

    fn expect_request_server_capabilities(&self) {
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

        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="{{USER_ID}}" type="get">
            <query xmlns='http://jabber.org/protocol/disco#info'/>
        </iq>"#
        );
        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" from="{{USER_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/disco#info">
            <identity category="account" type="registered" />
            <feature var="urn:xmpp:mam:2" />
            <feature var="urn:xmpp:mam:2#extended" />
            <feature var="urn:xmpp:sid:0" />
            <feature var="urn:xmpp:pep-vcard-conversion:0" />
            <identity category="pubsub" type="pep" />
            <feature var="http://jabber.org/protocol/pubsub" />
            <feature var="http://jabber.org/protocol/pubsub#publish" />
            <feature var="http://jabber.org/protocol/pubsub#presence-subscribe" />
            <feature var="http://jabber.org/protocol/pubsub#filtered-notifications" />
            <feature var="http://jabber.org/protocol/pubsub#delete-items" />
            <feature var="http://jabber.org/protocol/pubsub#retrieve-subscriptions" />
            <feature var="http://jabber.org/protocol/pubsub#purge-nodes" />
            <feature var="http://jabber.org/protocol/pubsub#publisher-affiliation" />
            <feature var="http://jabber.org/protocol/pubsub#item-ids" />
            <feature var="http://jabber.org/protocol/pubsub#retrieve-default" />
            <feature var="http://jabber.org/protocol/pubsub#config-node" />
            <feature var="http://jabber.org/protocol/pubsub#instant-nodes" />
            <feature var="http://jabber.org/protocol/pubsub#subscription-options" />
            <feature var="http://jabber.org/protocol/pubsub#config-node-max" />
            <feature var="http://jabber.org/protocol/pubsub#presence-notifications" />
            <feature var="http://jabber.org/protocol/pubsub#create-and-configure" />
            <feature var="http://jabber.org/protocol/pubsub#delete-nodes" />
            <feature var="http://jabber.org/protocol/pubsub#persistent-items" />
            <feature var="http://jabber.org/protocol/pubsub#modify-affiliations" />
            <feature var="http://jabber.org/protocol/pubsub#retrieve-items" />
            <feature var="http://jabber.org/protocol/pubsub#auto-subscribe" />
            <feature var="http://jabber.org/protocol/pubsub#access-presence" />
            <feature var="http://jabber.org/protocol/pubsub#member-affiliation" />
            <feature var="http://jabber.org/protocol/pubsub#auto-create" />
            <feature var="http://jabber.org/protocol/pubsub#subscribe" />
            <feature var="http://jabber.org/protocol/pubsub#create-nodes" />
            <feature var="http://jabber.org/protocol/pubsub#publish-options" />
            <feature var="http://jabber.org/protocol/pubsub#outcast-affiliation" />
            <feature var="http://jabber.org/protocol/pubsub#meta-data" />
            <feature var="http://jabber.org/protocol/pubsub#multi-items" />
            <feature var="http://jabber.org/protocol/pubsub#last-published" />
            <feature var="http://jabber.org/protocol/pubsub#retract-items" />
            <feature var="urn:ietf:params:xml:ns:vcard-4.0" />
          </query>
        </iq>
        "#
        );
    }

    fn expect_load_block_list(&self) {
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

    fn expect_publish_device(&self, existing_device_ids: impl IntoIterator<Item = DeviceId>) {
        let devices = existing_device_ids
            .into_iter()
            .map(|id| format!("<device id='{id}'/>"))
            .collect::<Vec<_>>()
            .join("\n");

        self.push_ctx([("EXISTING_DEVICES".into(), devices)].into());

        send!(
            self,
            r#"
            <iq xmlns='jabber:client' id="{{ID}}" type="set">
              <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                <publish node="eu.siacs.conversations.axolotl.devicelist">
                  <item id="current">
                    <list xmlns='eu.siacs.conversations.axolotl'>
                      {{EXISTING_DEVICES}}
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
