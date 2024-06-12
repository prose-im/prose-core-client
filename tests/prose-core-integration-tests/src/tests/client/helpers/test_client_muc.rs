// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use minidom::{Element, IntoAttributeValue};
use xmpp_parsers::mam::QueryId;

use prose_core_client::domain::rooms::services::impls::build_nickname;
use prose_core_client::domain::settings::models::SyncedRoomSettings;
use prose_core_client::domain::shared::models::AnonOccupantId;
use prose_core_client::domain::sidebar::models::BookmarkType;
use prose_core_client::dtos::{MucId, OccupantId, RoomEnvelope, RoomId, UserId};
use prose_core_client::ClientEvent;
use prose_xmpp::stanza::message::mam::ArchivedMessage;
use prose_xmpp::stanza::Message;
use prose_xmpp::TimeProvider;

use crate::{event, recv, send};

use super::TestClient;

pub struct JoinRoomStrategy {
    pub room_settings: Option<SyncedRoomSettings>,
    pub expect_catchup: Box<dyn FnOnce(&TestClient, &MucId)>,
}

impl Default for JoinRoomStrategy {
    fn default() -> Self {
        JoinRoomStrategy {
            room_settings: None,
            expect_catchup: Box::new(|client, room_id| client.expect_muc_catchup(room_id)),
        }
    }
}

impl JoinRoomStrategy {
    pub fn with_catch_up_handler(
        mut self,
        handler: impl FnOnce(&TestClient, &MucId) + 'static,
    ) -> Self {
        self.expect_catchup = Box::new(handler);
        self
    }
}

pub struct StartDMStrategy {
    pub room_settings: Option<SyncedRoomSettings>,
    pub expect_catchup: Box<dyn FnOnce(&TestClient, &UserId)>,
}

impl Default for StartDMStrategy {
    fn default() -> Self {
        Self {
            room_settings: None,
            expect_catchup: Box::new(|client, user_id| client.expect_catchup(user_id)),
        }
    }
}

impl StartDMStrategy {
    pub fn with_catch_up_handler(
        mut self,
        handler: impl FnOnce(&TestClient, &UserId) + 'static,
    ) -> Self {
        self.expect_catchup = Box::new(handler);
        self
    }
}

impl TestClient {
    pub fn build_occupant_id(&self, room_id: &MucId) -> OccupantId {
        let nickname = build_nickname(
            &self
                .client
                .connected_user_id()
                .expect("You're not connected")
                .into_user_id(),
        );
        room_id.occupant_id_with_nickname(nickname).unwrap()
    }

    pub async fn join_room(
        &self,
        room_id: MucId,
        anon_occupant_id: impl Into<AnonOccupantId>,
    ) -> Result<()> {
        self.join_room_with_strategy(room_id, anon_occupant_id, Default::default())
            .await
    }

    pub async fn join_room_with_strategy(
        &self,
        room_id: MucId,
        anon_occupant_id: impl Into<AnonOccupantId>,
        strategy: JoinRoomStrategy,
    ) -> Result<()> {
        let occupant_id = self.build_occupant_id(&room_id);
        let room_name = "general";
        let anon_occupant_id = anon_occupant_id.into();

        self.push_ctx(
            [
                ("OCCUPANT_ID".into(), occupant_id.to_string()),
                ("ROOM_ID".into(), room_id.to_string()),
                ("ROOM_NAME".into(), room_name.to_string()),
                ("ANON_OCCUPANT_ID".into(), anon_occupant_id.to_string()),
            ]
            .into(),
        );

        send!(
            self,
            r#"
        <presence xmlns='jabber:client' to="{{OCCUPANT_ID}}">
            <show>chat</show>
            <x xmlns='http://jabber.org/protocol/muc'>
              <history maxstanzas="0" />
            </x>
            <c xmlns='http://jabber.org/protocol/caps' hash="sha-1" node="https://prose.org" ver="{{CAPS_HASH}}"/>
        </presence>
        "#
        );

        recv!(
            self,
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
            self,
            r#"
        <message xmlns="jabber:client" from="{{OCCUPANT_ID}}" type="groupchat">
          <subject />
        </message>
        "#
        );

        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="{{ROOM_ID}}" type="get">
            <query xmlns='http://jabber.org/protocol/disco#info'/>
        </iq>"#
        );
        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/disco#info">
            <feature var="muc_persistent" />
            <feature var="http://jabber.org/protocol/muc#request" />
            <feature var="urn:xmpp:mam:2" />
            <feature var="urn:xmpp:mam:2#extended" />
            <feature var="urn:xmpp:sid:0" />
            <feature var="muc_public" />
            <feature var="muc_unmoderated" />
            <feature var="muc_unsecured" />
            <feature var="muc_open" />
            <feature var="jabber:iq:register" />
            <feature var="urn:xmpp:occupant-id:0" />
            <feature var="http://jabber.org/protocol/muc" />
            <feature var="http://jabber.org/protocol/muc#stable_id" />
            <feature var="http://jabber.org/protocol/muc#self-ping-optimization" />
            <identity category="conference" name="{{ROOM_NAME}}" type="text" />
            <feature var="muc_nonanonymous" />
            <x xmlns="jabber:x:data" type="result">
              <field type="hidden" var="FORM_TYPE">
                <value>http://jabber.org/protocol/muc#roominfo</value>
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
                <value>1</value>
              </field>
              <field type="boolean" var="muc#roomconfig_changesubject">
                <value>1</value>
              </field>
              <field label="Description" type="text-single" var="muc#roominfo_description">
                <value />
              </field>
              <field label="Title" type="text-single" var="muc#roomconfig_roomname">
                <value>{{ROOM_NAME}}</value>
              </field>
            </x>
          </query>
        </iq>
        "#
        );

        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="{{ROOM_ID}}" type="get">
            <query xmlns='http://jabber.org/protocol/muc#admin'>
                <item xmlns='http://jabber.org/protocol/muc#user' affiliation="owner"/>
            </query>
        </iq>
        "#
        );
        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
          <query xmlns="http://jabber.org/protocol/muc#admin">
            <item affiliation="owner" jid="user@prose.org" />
          </query>
        </iq>
        "#
        );

        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="{{ROOM_ID}}" type="get">
            <query xmlns='http://jabber.org/protocol/muc#admin'>
                <item xmlns='http://jabber.org/protocol/muc#user' affiliation="member"/>
            </query>
        </iq>
        "#
        );
        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
            <query xmlns="http://jabber.org/protocol/muc#admin" />
        </iq>
        "#
        );

        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="{{ROOM_ID}}" type="get">
            <query xmlns='http://jabber.org/protocol/muc#admin'>
                <item xmlns='http://jabber.org/protocol/muc#user' affiliation="admin"/>
            </query>
        </iq>
        "#
        );
        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" from="{{ROOM_ID}}" id="{{ID}}" type="result">
            <query xmlns="http://jabber.org/protocol/muc#admin" />
        </iq>
        "#
        );

        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="user@prose.org" type="get">
            <vcard xmlns='urn:ietf:params:xml:ns:vcard-4.0'/>
        </iq>"#
        );
        recv!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" type="result">
            <vcard xmlns='urn:ietf:params:xml:ns:vcard-4.0'>
                <adr>
                    <country>Germany</country>
                    <locality>Berlin</locality>
                </adr>
                <email><text>user@prose.org</text></email>
                <nickname><text>Joe</text></nickname>
            </vcard>
        </iq>"#
        );

        self.expect_load_settings(room_id.clone(), strategy.room_settings);
        (strategy.expect_catchup)(&self, &room_id);

        self.expect_set_bookmark(
            &RoomId::Muc(room_id.clone()),
            room_name,
            BookmarkType::PublicChannel,
        );

        event!(self, ClientEvent::SidebarChanged);

        self.pop_ctx();

        self.rooms.join_room(&room_id, None).await?;

        Ok(())
    }

    pub async fn start_dm(&self, user_id: UserId) -> Result<RoomEnvelope> {
        self.start_dm_with_strategy(user_id, Default::default())
            .await
    }

    pub async fn start_dm_with_strategy(
        &self,
        user_id: UserId,
        strategy: StartDMStrategy,
    ) -> Result<RoomEnvelope> {
        self.push_ctx([("OTHER_USER_ID".into(), user_id.to_string())].into());

        send!(
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{OTHER_USER_ID}}" type="get">
            <vcard xmlns="urn:ietf:params:xml:ns:vcard-4.0" />
        </iq>
        "#
        );

        recv!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="error">
              <error type="cancel">
                <item-not-found xmlns="urn:ietf:params:xml:ns:xmpp-stanzas" />
              </error>
            </iq>
            "#
        );

        send!(
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{OTHER_USER_ID}}" type="get">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <items max_items="1" node="urn:xmpp:avatar:metadata" />
          </pubsub>
        </iq>
        "#
        );

        recv!(
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="error">
          <error type="cancel">
            <item-not-found xmlns="urn:ietf:params:xml:ns:xmpp-stanzas" />
          </error>
        </iq>
        "#
        );

        self.expect_load_settings(user_id.clone(), strategy.room_settings);
        (strategy.expect_catchup)(&self, &user_id);

        self.expect_set_bookmark(
            &RoomId::User(user_id.clone()),
            user_id.formatted_username(),
            BookmarkType::DirectMessage,
        );

        event!(self, ClientEvent::SidebarChanged);

        self.rooms.start_conversation(&[user_id.clone()]).await?;

        Ok(self.get_room(user_id).await)
    }

    pub fn expect_set_bookmark(
        &self,
        room_id: &RoomId,
        name: impl Into<String>,
        kind: BookmarkType,
    ) {
        self.push_ctx(
            [
                ("ROOM_ID".into(), room_id.to_string()),
                ("BOOKMARK_NAME".into(), name.into()),
                ("BOOKMARK_TYPE".into(), kind.into_attribute_value().unwrap()),
            ]
            .into(),
        );

        send!(
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" type="set">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="https://prose.org/protocol/bookmark">
              <item id="{{ROOM_ID}}">
                <bookmark xmlns="https://prose.org/protocol/bookmark" jid="{{ROOM_ID}}" name="{{BOOKMARK_NAME}}" sidebar="1" type="{{BOOKMARK_TYPE}}" />
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
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_ID}}" type="result">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="https://prose.org/protocol/bookmark">
              <item id="{{ROOM_ID}}" />
            </publish>
          </pubsub>
        </iq>
        "#
        );

        self.pop_ctx();
    }

    pub fn expect_publish_settings(&self, settings: SyncedRoomSettings) {
        self.push_ctx(
            [
                ("ROOM_ID".into(), settings.room_id.to_string()),
                (
                    "ROOM_SETTINGS".into(),
                    String::from(&Element::from(settings)),
                ),
            ]
            .into(),
        );

        send!(
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" type="set">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="https://prose.org/protocol/room_settings">
              <item id="{{ROOM_ID}}">
                {{ROOM_SETTINGS}}
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
            self,
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
    }

    pub fn expect_muc_catchup(&self, room_id: &MucId) {
        self.expect_muc_catchup_with_config(
            room_id,
            self.time_provider.now() - Duration::seconds(self.app_config.max_catchup_duration_secs),
            None,
        )
    }

    pub fn expect_muc_catchup_with_config(
        &self,
        room_id: &MucId,
        start: DateTime<Utc>,
        messages: impl IntoIterator<Item = ArchivedMessage>,
    ) {
        self.push_ctx(
            [
                ("ROOM_ID".into(), room_id.to_string()),
                ("CATCHUP_START".into(), start.to_rfc3339()),
            ]
            .into(),
        );

        send!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID:2}}" to="{{ROOM_ID}}" type="set">
              <query xmlns="urn:xmpp:mam:2" queryid="{{ID:1}}">
                <x xmlns="jabber:x:data" type="submit">
                  <field type="hidden" var="FORM_TYPE">
                    <value>urn:xmpp:mam:2</value>
                  </field>
                  <field var="start">
                    <value>{{CATCHUP_START}}</value>
                  </field>
                </x>
                <set xmlns="http://jabber.org/protocol/rsm">
                  <max>100</max>
                </set>
              </query>
            </iq>
            "#
        );

        let query_id = QueryId(self.id_provider.id_with_offset(1));

        for mut archived_message in messages.into_iter() {
            archived_message.query_id = Some(query_id.clone());

            let message = Message::new().set_archived_message(archived_message);
            self.receive_element(Element::from(message), file!(), line!());
        }

        recv!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result">
                <fin xmlns="urn:xmpp:mam:2" complete="true">
                    <set xmlns="http://jabber.org/protocol/rsm" />
                </fin>
            </iq>
            "#
        );

        self.pop_ctx();
    }

    pub fn expect_load_settings(
        &self,
        room_id: impl Into<RoomId>,
        settings: Option<SyncedRoomSettings>,
    ) {
        self.push_ctx([("ROOM_ID".into(), room_id.into().to_string())].into());

        send!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" type="get">
              <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <items node="https://prose.org/protocol/room_settings">
                  <item id="{{ROOM_ID}}" />
                </items>
              </pubsub>
             </iq>
            "#
        );

        if let Some(settings) = settings {
            self.push_ctx(
                [(
                    "ROOM_SETTINGS".into(),
                    String::from(&Element::from(settings)),
                )]
                .into(),
            );

            recv!(
                self,
                r#"
                <iq xmlns="jabber:client" id="{{ID}}" type="result">
                  <pubsub xmlns="http://jabber.org/protocol/pubsub">
                    <items node="https://prose.org/protocol/room_settings">
                      <item id="{{ROOM_ID}}">
                        {{ROOM_SETTINGS}}
                      </item>
                    </items>
                  </pubsub>
                </iq>
                "#
            );

            self.pop_ctx();
        } else {
            recv!(
                self,
                r#"
                <iq xmlns="jabber:client" id="{{ID}}" type="error">
                  <pubsub xmlns="http://jabber.org/protocol/pubsub">
                    <items node="https://prose.org/protocol/room_settings">
                      <item id="{{ROOM_ID}}" />
                    </items>
                  </pubsub>
                  <error type="cancel">
                    <item-not-found xmlns="urn:ietf:params:xml:ns:xmpp-stanzas"/>
                  </error>
                </iq>
                "#
            );
        }

        self.pop_ctx();
    }

    pub fn expect_catchup(&self, room_id: &UserId) {
        self.expect_catchup_with_config(room_id, None);
    }

    pub fn expect_catchup_with_config(
        &self,
        room_id: &UserId,
        messages: impl IntoIterator<Item = ArchivedMessage>,
    ) {
        let start =
            self.time_provider.now() - Duration::seconds(self.app_config.max_catchup_duration_secs);

        self.push_ctx(
            [
                ("ROOM_ID".into(), room_id.to_string()),
                ("START".into(), start.to_rfc3339()),
            ]
            .into(),
        );

        send!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID:2}}" type="set">
              <query xmlns="urn:xmpp:mam:2" queryid="{{ID:1}}">
                <x xmlns="jabber:x:data" type="submit">
                  <field type="hidden" var="FORM_TYPE">
                    <value>urn:xmpp:mam:2</value>
                  </field>
                  <field var="start">
                    <value>{{START}}</value>
                  </field>
                  <field var="with">
                    <value>{{ROOM_ID}}</value>
                  </field>
                </x>
                <set xmlns="http://jabber.org/protocol/rsm">
                  <max>100</max>
                </set>
              </query>
            </iq>
            "#
        );

        let query_id = QueryId(self.id_provider.id_with_offset(1));

        for mut archived_message in messages.into_iter() {
            archived_message.query_id = Some(query_id.clone());

            let message = Message::new().set_archived_message(archived_message);
            self.receive_element(Element::from(message), file!(), line!());
        }

        recv!(
            self,
            r#"
            <iq xmlns="jabber:client" id="{{ID}}" to="{{USER_RESOURCE_ID}}" type="result">
                <fin xmlns="urn:xmpp:mam:2" complete="true">
                    <set xmlns="http://jabber.org/protocol/rsm" />
                </fin>
            </iq>
            "#
        );

        self.pop_ctx();
    }
}
