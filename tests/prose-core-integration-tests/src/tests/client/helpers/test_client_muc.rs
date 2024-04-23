// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::domain::rooms::services::impls::build_nickname;
use prose_core_client::dtos::{MucId, RoomEnvelope, UserId};

use crate::{recv, send};

use super::TestClient;

impl TestClient {
    pub async fn perform_join_room(&self, room_id: MucId) -> Result<()> {
        let nickname = build_nickname(
            &self
                .client
                .connected_user_id()
                .expect("You're not connected")
                .into_user_id(),
        );
        let occupant_id = room_id.occupant_id_with_nickname(nickname)?;

        self.push_ctx(
            [
                ("OCCUPANT_ID".into(), occupant_id.to_string()),
                ("ROOM_ID".into(), room_id.to_string()),
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
            <c xmlns='http://jabber.org/protocol/caps' hash="sha-1" node="https://prose.org" ver="6F3DapJergay3XYdZEtLkCjrPpc="/>
        </presence>
        "#
        );

        recv!(
            self,
            r#"
        <presence xmlns="jabber:client" from="{{OCCUPANT_ID}}" xml:lang="en">
          <show>chat</show>
          <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="https://prose.org" ver="6F3DapJergay3XYdZEtLkCjrPpc=" />
          <occupant-id xmlns="urn:xmpp:occupant-id:0" id="LlY4x7k0T+udxUmRfaIuYJB1pzlFu4yEziE7hzxaeYI=" />
          <x xmlns="http://jabber.org/protocol/muc#user">
            <status code="100" />
            <item affiliation="owner" jid="m@nsm.chat/tnFAvzAb" role="moderator" />
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
            <identity category="conference" name="general" type="text" />
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
                <value>general</value>
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

        send!(
            self,
            r#"
        <iq xmlns='jabber:client' id="{{ID}}" type="set">
            <pubsub xmlns='http://jabber.org/protocol/pubsub'>
                <publish node="https://prose.org/protocol/bookmark">
                    <item id="{{ROOM_ID}}">
                        <bookmark
                            xmlns='https://prose.org/protocol/bookmark'
                            jid="{{ROOM_ID}}"
                            name="general"
                            sidebar="1"
                            type="public-channel"
                        />
                    </item>
                </publish>
                <publish-options>
                    <x xmlns='jabber:x:data' type="submit">
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
        </iq>"#
        );
        recv!(
            self,
            r#"<iq xmlns="jabber:client" id="{{ID}}" type="result" />"#
        );

        self.pop_ctx();

        self.rooms.join_room(&room_id, None).await?;

        Ok(())
    }

    pub async fn perform_start_dm(&self, user_id: UserId) -> Result<RoomEnvelope> {
        send!(
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" to="them@prose.org" type="get">
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
        <iq xmlns="jabber:client" id="{{ID}}" to="them@prose.org" type="get">
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

        send!(
            self,
            r#"
        <iq xmlns="jabber:client" id="{{ID}}" type="set">
          <pubsub xmlns="http://jabber.org/protocol/pubsub">
            <publish node="https://prose.org/protocol/bookmark">
              <item id="them@prose.org">
                <bookmark xmlns="https://prose.org/protocol/bookmark" jid="them@prose.org" name="Them" sidebar="1" type="dm" />
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
              <item id="them@prose.org" />
            </publish>
          </pubsub>
        </iq>
        "#
        );

        self.rooms.start_conversation(&[user_id.clone()]).await?;

        Ok(self.get_room(user_id).await)
    }
}
