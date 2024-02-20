// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::dtos::{RoomId, SendMessageRequest, UserId};
use prose_core_client::{room_id, user_id};
use prose_proc_macros::mt_test;

use crate::tests::helpers::TestClient;

#[mt_test]
async fn test_joins_room() -> Result<()> {
    let client = TestClient::new().await;

    client.send(
        r#"
    <iq xmlns='jabber:client' id="{{ID}}" type="get">
        <query xmlns='jabber:iq:roster'/>
    </iq>"#,
    );
    client.receive(
        r#"
    <iq xmlns="jabber:client" id="{{ID}}" type="result">
        <query xmlns="jabber:iq:roster" ver="1" />
    </iq>
    "#,
    );

    client.send(r#"
    <presence xmlns='jabber:client'>
        <show>chat</show>
        <c xmlns='http://jabber.org/protocol/caps' hash="sha-1" node="https://prose.org" ver="dWkYhmO8yFRrk+2G6R324ES7G9E=" />
    </presence>"#);

    client.send(
        r#"
    <iq xmlns='jabber:client' id="{{ID}}" type="set">
        <enable xmlns='urn:xmpp:carbons:2'/>
    </iq>"#,
    );
    client.receive(r#"<iq xmlns="jabber:client" id="{{ID}}" type="result" />"#);

    client.send(
        r#"
    <iq xmlns='jabber:client' id="{{ID}}" to="prose.org" type="get">
        <query xmlns='http://jabber.org/protocol/disco#items'/>
    </iq>"#,
    );
    client.receive(
        r#"
    <iq xmlns="jabber:client" from="nsm.chat" id="{{ID}}" type="result">
        <query xmlns="http://jabber.org/protocol/disco#items">
            <item jid="conference.prose.org" name="Chatrooms" />
            <item jid="upload.prose.org" name="HTTP File Upload" />
        </query>
    </iq>
    "#,
    );

    client.send(
        r#"
    <iq xmlns='jabber:client' id="{{ID}}" to="conference.prose.org" type="get">
        <query xmlns='http://jabber.org/protocol/disco#info'/>
    </iq>"#,
    );
    client.receive(
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

    client.send(
        r#"
    <iq xmlns='jabber:client' id="{{ID}}" to="upload.prose.org" type="get">
        <query xmlns='http://jabber.org/protocol/disco#info'/>
    </iq>"#,
    );
    client.receive(
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

    client.send(
        r#"
    <iq xmlns='jabber:client' id="{{ID}}" type="get">
        <blocklist xmlns='urn:xmpp:blocking'/>
    </iq>"#,
    );
    client.receive(
        r#"
    <iq xmlns="jabber:client" id="{{ID}}" type="result">
      <blocklist xmlns="urn:xmpp:blocking" />
    </iq>
    "#,
    );

    client
        .connect(&user_id!("user@prose.org"), "secret")
        .await?;

    client.send(r#"
    <presence xmlns='jabber:client' to="room@conference.prose.org/user#1ed8798">
        <show>chat</show>
        <x xmlns='http://jabber.org/protocol/muc'/>
        <c xmlns='http://jabber.org/protocol/caps' hash="sha-1" node="https://prose.org" ver="dWkYhmO8yFRrk+2G6R324ES7G9E="/>
    </presence>"#);
    client.receive(r#"
    <presence xmlns="jabber:client" from="room@conference.prose.org/user#1ed8798" xml:lang="en">
      <show>chat</show>
      <c xmlns="http://jabber.org/protocol/caps" hash="sha-1" node="https://prose.org" ver="dWkYhmO8yFRrk+2G6R324ES7G9E=" />
      <occupant-id xmlns="urn:xmpp:occupant-id:0" id="LlY4x7k0T+udxUmRfaIuYJB1pzlFu4yEziE7hzxaeYI=" />
      <x xmlns="http://jabber.org/protocol/muc#user">
        <status code="100" />
        <item affiliation="owner" jid="m@nsm.chat/tnFAvzAb" role="moderator" />
        <status code="110" />
      </x>
    </presence>
    "#);
    client.receive(
        r#"
    <message xmlns="jabber:client" from="room@conference.prose.org/user#1ed8798" type="groupchat">
      <subject />
    </message>
    "#,
    );

    client.send(
        r#"
    <iq xmlns='jabber:client' id="{{ID}}" to="room@conference.prose.org" type="get">
        <query xmlns='http://jabber.org/protocol/disco#info'/>
    </iq>"#,
    );
    client.receive(r#"
    <iq xmlns="jabber:client" from="room@conference.prose.org" id="{{ID}}" type="result">
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
    "#);

    client.send(
        r#"
    <iq xmlns='jabber:client' id="{{ID}}" to="room@conference.prose.org" type="get">
        <query xmlns='http://jabber.org/protocol/muc#admin'>
            <item xmlns='http://jabber.org/protocol/muc#user' affiliation="owner"/>
        </query>
    </iq>
    "#,
    );
    client.receive(
        r#"
    <iq xmlns="jabber:client" from="room@conference.prose.org" id="{{ID}}" type="result">
      <query xmlns="http://jabber.org/protocol/muc#admin">
        <item affiliation="owner" jid="user@prose.org" />
      </query>
    </iq>
    "#,
    );

    client.send(
        r#"
    <iq xmlns='jabber:client' id="{{ID}}" to="room@conference.prose.org" type="get">
        <query xmlns='http://jabber.org/protocol/muc#admin'>
            <item xmlns='http://jabber.org/protocol/muc#user' affiliation="member"/>
        </query>
    </iq>
    "#,
    );
    client.receive(
        r#"
    <iq xmlns="jabber:client" from="room@conference.prose.org" id="{{ID}}" type="result">
        <query xmlns="http://jabber.org/protocol/muc#admin" />
    </iq>
    "#,
    );

    client.send(
        r#"
    <iq xmlns='jabber:client' id="{{ID}}" to="room@conference.prose.org" type="get">
        <query xmlns='http://jabber.org/protocol/muc#admin'>
            <item xmlns='http://jabber.org/protocol/muc#user' affiliation="admin"/>
        </query>
    </iq>
    "#,
    );
    client.receive(
        r#"
    <iq xmlns="jabber:client" from="room@conference.prose.org" id="{{ID}}" type="result">
        <query xmlns="http://jabber.org/protocol/muc#admin" />
    </iq>
    "#,
    );

    client.send(
        r#"
        <iq xmlns='jabber:client' id="{{ID}}" to="user@prose.org" type="get">
            <vcard xmlns='urn:ietf:params:xml:ns:vcard-4.0'/>
        </iq>"#,
    );
    client.receive(
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
    </iq>"#,
    );

    client.send(
        r#"
    <iq xmlns='jabber:client' id="{{ID}}" type="set">
        <pubsub xmlns='http://jabber.org/protocol/pubsub'>
            <publish node="https://prose.org/protocol/bookmark">
                <item id="room@conference.prose.org">
                    <bookmark 
                        xmlns='https://prose.org/protocol/bookmark' 
                        jid="room@conference.prose.org" 
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
    </iq>"#,
    );
    client.receive(r#"<iq xmlns="jabber:client" id="{{ID}}" type="result" />"#);

    client
        .rooms
        .join_room(&room_id!("room@conference.prose.org"), None)
        .await?;

    Ok(())
}
