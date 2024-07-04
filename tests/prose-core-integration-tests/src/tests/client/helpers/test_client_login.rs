// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::BareJid;
use minidom::Element;
use xmpp_parsers::pubsub;
use xmpp_parsers::roster::Item as RosterItem;

use prose_core_client::domain::user_info::models::UserProfile;
use prose_core_client::dtos::{Bookmark, DeviceBundle, DeviceId, UserId};
use prose_core_client::{ClientEvent, ConnectionEvent, Secret};
use prose_xmpp::stanza::vcard4::Nickname;
use prose_xmpp::stanza::VCard4;
use prose_xmpp::IDProvider;

use crate::{event, recv, send};

use super::TestClient;

pub struct LoginStrategy {
    pub device_bundles: Vec<(DeviceId, DeviceBundle)>,
    pub offline_messages: Vec<prose_xmpp::stanza::Message>,
    pub bookmarks_handler: Box<dyn FnOnce(&TestClient)>,
    pub user_vcard: Option<VCard4>,
    pub roster_items: Vec<RosterItem>,
}

impl Default for LoginStrategy {
    fn default() -> Self {
        Self {
            device_bundles: vec![],
            offline_messages: vec![],
            bookmarks_handler: Box::new(|client| client.expect_load_bookmarks(None)),
            user_vcard: Some(VCard4 {
                adr: vec![],
                email: vec![],
                fn_: vec![],
                n: vec![],
                impp: vec![],
                nickname: vec![Nickname {
                    value: "Jane Doe".to_string(),
                }],
                note: vec![],
                org: vec![],
                role: vec![],
                tel: vec![],
                title: vec![],
                url: vec![],
            }),
            roster_items: vec![],
        }
    }
}

impl LoginStrategy {
    pub fn with_device_bundles(
        mut self,
        device_bundles: impl IntoIterator<Item = (DeviceId, DeviceBundle)>,
    ) -> Self {
        self.device_bundles = device_bundles.into_iter().collect::<Vec<_>>();
        self
    }

    pub fn with_offline_messages(
        mut self,
        offline_messages: impl IntoIterator<Item = prose_xmpp::stanza::Message>,
    ) -> Self {
        self.offline_messages = offline_messages.into_iter().collect::<Vec<_>>();
        self
    }

    pub fn with_bookmarks_handler(mut self, handler: impl FnOnce(&TestClient) + 'static) -> Self {
        self.bookmarks_handler = Box::new(handler);
        self
    }

    pub fn with_roster_items(mut self, items: impl IntoIterator<Item = RosterItem>) -> Self {
        self.roster_items = items.into_iter().collect();
        self
    }

    pub fn with_user_vcard(mut self, vcard4: Option<VCard4>) -> Self {
        self.user_vcard = vcard4;
        self
    }
}

impl TestClient {
    pub async fn expect_login(&self, user: UserId, password: impl AsRef<str>) -> Result<()> {
        self.expect_login_with_strategy(user, password, LoginStrategy::default())
            .await
    }

    pub async fn expect_login_with_strategy(
        &self,
        user: UserId,
        password: impl AsRef<str>,
        strategy: LoginStrategy,
    ) -> Result<()> {
        let last_nickname = self.get_ctx("USER_NICKNAME");

        let nickname = strategy
            .user_vcard
            .clone()
            .and_then(|vcard| UserProfile::try_from(vcard).unwrap().nickname)
            .unwrap_or_else(|| "You forgot to set a nickname".to_string());

        self.push_ctx([
            ("USER_ID", user.to_string()),
            (
                "USER_RESOURCE_ID",
                format!("{}/{}", user.to_string(), self.short_id_provider.new_id()),
            ),
            (
                "SERVER_ID",
                BareJid::from_parts(None, &user.as_ref().domain()).to_string(),
            ),
            ("CAPS_HASH", "IypOfhCkiLIruAabdFj0jeESeqc=".to_string()),
            ("USER_NICKNAME", nickname.clone()),
        ]);

        self.expect_load_roster(strategy.roster_items);

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

        if !strategy.offline_messages.is_empty() {
            for message in strategy.offline_messages {
                self.receive_element(message, file!(), line!());
            }
            self.receive_next().await;
        }

        self.expect_request_server_capabilities();

        send!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" to="{{SERVER_ID}}" type="get">
              <time xmlns="urn:xmpp:time" />
            </iq>
            "#
        );

        // Let's not return a server time. otherwise timestamps for received messages will be
        // off by a few milliseconds.
        recv!(
            self,
            r#"
            <iq xmlns="jabber:client" from="{{SERVER_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="error">
              <time xmlns="urn:xmpp:time" />
              <error type="cancel">
                <feature-not-implemented xmlns="urn:ietf:params:xml:ns:xmpp-stanzas" />
              </error>
            </iq>
            "#
        );

        self.expect_load_block_list();

        self.expect_load_device_list(
            &user,
            strategy
                .device_bundles
                .clone()
                .into_iter()
                .map(|(id, _)| id),
        );
        self.expect_publish_device(strategy.device_bundles.into_iter().map(|(id, _)| id));
        self.expect_publish_initial_device_bundle();

        event!(
            self,
            ClientEvent::ConnectionStatusChanged {
                event: ConnectionEvent::Connect,
            }
        );

        event!(self, ClientEvent::AccountInfoChanged);

        self.connect(&user, Secret::new(password.as_ref().to_string()))
            .await?;

        if let Some(vcard) = strategy.user_vcard {
            self.push_ctx([("VCARD", String::from(&Element::from(vcard)))]);
            recv!(
                self,
                r#"
                <message xmlns="jabber:client" from="{{USER_ID}}" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="headline">
                  <event xmlns="http://jabber.org/protocol/pubsub#event">
                    <items node="urn:ietf:params:xml:ns:vcard-4.0">
                      <item id="{{USER_ID}}" publisher="{{USER_ID}}">
                        {{VCARD}}
                      </item>
                    </items>
                  </event>
                </message>
                "#
            );

            if Some(nickname) != last_nickname {
                event!(
                    self,
                    ClientEvent::ContactChanged {
                        ids: vec![self.connected_user_id().unwrap().into_user_id()]
                    }
                );
                event!(self, ClientEvent::AccountInfoChanged);
            }

            self.receive_next().await;
            self.pop_ctx();
        }

        (strategy.bookmarks_handler)(self);

        self.pop_ctx();

        self.rooms.start_observing_rooms().await?;

        Ok(())
    }
}

impl TestClient {
    fn expect_load_roster(&self, items: Vec<RosterItem>) {
        let items = items
            .into_iter()
            .map(|item| String::from(&Element::from(item)))
            .collect::<Vec<_>>()
            .join("\n");

        self.push_ctx([("ROSTER_ITEMS", items)]);
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
            <query xmlns="jabber:iq:roster" ver="1">
                {{ROSTER_ITEMS}}
            </query>
        </iq>
        "#
        );
        self.pop_ctx();
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

    pub fn expect_load_bookmarks(&self, bookmarks: impl IntoIterator<Item = Bookmark>) {
        send!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" type="get">
              <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <items node="https://prose.org/protocol/bookmark" />
              </pubsub>
            </iq>
            "#
        );

        let bookmarks = bookmarks
            .into_iter()
            .map(|bookmark| {
                String::from(&Element::from(pubsub::pubsub::Item(pubsub::Item {
                    id: Some(pubsub::ItemId(bookmark.jid.to_string())),
                    publisher: None,
                    payload: Some(Element::from(bookmark.clone())),
                })))
            })
            .collect::<Vec<_>>()
            .join("\n");

        self.push_ctx([("BOOKMARKS", bookmarks)]);

        recv!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" type="result">
              <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <items node="https://prose.org/protocol/bookmark">
                    {{BOOKMARKS}}
                </items>
              </pubsub>
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

        self.push_ctx([("EXISTING_DEVICES", devices)]);

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
