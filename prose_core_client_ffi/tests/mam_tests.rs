use jid::BareJid;
use prose_core_client_ffi::test_helpers::mocks::HandlerBucketExt;
use prose_core_client_ffi::test_helpers::StrExt;
use prose_core_client_ffi::{
    Account, Result, XMPPDelay, XMPPForwardedMessage, XMPPMAMDefaultBehavior, XMPPMAMPreferences,
    XMPPMessage,
};
use std::str::FromStr;

#[test]
fn test_loads_archiving_preferences() -> Result<()> {
    let (account, handlers, stanzas, observer) = Account::connected();

    account.mam.load_archiving_preferences()?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"<iq id="id_1" type="get"><prefs xmlns="urn:xmpp:mam:2"/></iq>"#.to_string()
    );

    let expected_preferences = XMPPMAMPreferences::new(
        XMPPMAMDefaultBehavior::Roster,
        vec![
            BareJid::from_str("a@prose.org").unwrap(),
            BareJid::from_str("c@prose.org").unwrap(),
        ],
        vec![BareJid::from_str("b@prose.org").unwrap()],
    );

    observer
        .lock()
        .unwrap()
        .expect_did_receive_archiving_preferences(|arg| arg.partial_eq(expected_preferences))
        .times(1)
        .returns(());

    handlers.send_stanza_str(
        r#"
        <iq type='result' id='id_1'>
        <prefs xmlns="urn:xmpp:mam:2" default="roster">
            <always><jid>a@prose.org</jid><jid>c@prose.org</jid></always>
            <never><jid>b@prose.org</jid></never>
        </prefs>
        </iq>"#,
    );

    Ok(())
}

#[test]
fn test_sets_archiving_preferences() -> Result<()> {
    let (account, _, stanzas, _) = Account::connected();

    let preferences = XMPPMAMPreferences::new(
        XMPPMAMDefaultBehavior::Roster,
        vec![],
        vec![BareJid::from_str("b@prose.org").unwrap()],
    );

    account.mam.set_archiving_preferences(&preferences)?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"<iq id="id_1" type="set"><prefs default="roster" xmlns="urn:xmpp:mam:2"><always/><never><jid>b@prose.org</jid></never></prefs></iq>"#.to_string()
    );

    Ok(())
}

#[test]
fn test_loads_messages() -> Result<()> {
    let (account, handlers, stanzas, observer) = Account::connected();

    account.mam.load_messages_in_chat(
        "my-request",
        &BareJid::from_str("b@prose.org").unwrap(),
        None,
    )?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"
        <iq id="my-request" type="set">
            <query queryid="id_1" xmlns="urn:xmpp:mam:2">
                <x xmlns="jabber:x:data" type="submit">
                    <field type="hidden" var="FORM_TYPE">
                        <value>urn:xmpp:mam:2</value>
                    </field>
                    <field var="with"><value>b@prose.org</value></field>
                </x>
                <flip-page/>
            </query>
        </iq>"#
            .to_xml_result_string()
    );

    let expected_messages = vec![
        XMPPForwardedMessage::new(
            XMPPMessage::new_chat_message(
                BareJid::from_str("a@prose.org").unwrap(),
                BareJid::from_str("b@prose.org").unwrap(),
                None,
                "message 1",
                None,
            ),
            Some(XMPPDelay::new(1657810800, None)),
        ),
        XMPPForwardedMessage::new(
            XMPPMessage::new_chat_message(
                BareJid::from_str("b@prose.org").unwrap(),
                BareJid::from_str("a@prose.org").unwrap(),
                None,
                "message 3",
                None,
            ),
            Some(XMPPDelay::new(1657810920, None)),
        ),
    ];

    observer
        .lock()
        .unwrap()
        .expect_did_receive_messages_in_chat(
            |arg| arg.partial_eq("my-request"),
            |arg| arg.partial_eq(BareJid::from_str("b@prose.org").unwrap()),
            |arg| arg.partial_eq(expected_messages),
            |arg| arg.partial_eq(true),
        )
        .times(1)
        .returns(());

    handlers.send_stanza_str(
        r#"
        <message to="a@prose.org">
            <result queryid="id_1" xmlns="urn:xmpp:mam:2">
                <forwarded xmlns="urn:xmpp:forward:0">
                    <delay xmlns="urn:xmpp:delay" stamp="2022-07-14T15:00:00Z"/>
                    <message to="b@prose.org" from="a@prose.org" type="chat">
                        <body>message 1</body>
                    </message>
                </forwarded>
            </result>
        </message>
  "#,
    );

    // This one should be ignored since the queryid doesn't match.
    handlers.send_stanza_str(
        r#"
        <message to="a@prose.org">
            <result queryid="id_2" xmlns="urn:xmpp:mam:2">
                <forwarded xmlns="urn:xmpp:forward:0">
                    <delay xmlns="urn:xmpp:delay" stamp="2022-07-14T15:01:00Z"/>
                    <message to="b@prose.org" from="a@prose.org" type="chat">
                        <body>message 2</body>
                    </message>
                </forwarded>
            </result>
        </message>
  "#,
    );

    handlers.send_stanza_str(
        r#"
        <message to="a@prose.org">
            <result queryid="id_1" xmlns="urn:xmpp:mam:2">
                <forwarded xmlns="urn:xmpp:forward:0">
                    <delay xmlns="urn:xmpp:delay" stamp="2022-07-14T15:02:00Z"/>
                    <message to="a@prose.org" from="b@prose.org" type="chat">
                        <body>message 3</body>
                    </message>
                </forwarded>
            </result>
        </message>
  "#,
    );

    handlers.send_stanza_str(
        r#"
      <iq type="result" to="a@prose.org">
        <fin queryid="id_1" xmlns="urn:xmpp:mam:2" complete="true">
            <set xmlns="http://jabber.org/protocol/rsm">
                <first>message-id-1</first>
                <last>message-id-2</last>
            </set>
        </fin>
      </iq>
      "#,
    );

    // The second fin should have no effect.
    handlers.send_stanza_str(
        r#"
      <iq type="result" to="a@prose.org">
        <fin queryid="id_1" xmlns="urn:xmpp:mam:2" complete="true">
            <set xmlns="http://jabber.org/protocol/rsm">
                <first>message-id-1</first>
                <last>message-id-2</last>
            </set>
        </fin>
      </iq>
      "#,
    );

    Ok(())
}

#[test]
fn test_loads_messages_before() -> Result<()> {
    let (account, _, stanzas, _) = Account::connected();

    account.mam.load_messages_in_chat(
        "my-request",
        &BareJid::from_str("b@prose.org").unwrap(),
        Some("09af3-cc343-b409f".into()),
    )?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"
        <iq id="my-request" type="set">
            <query queryid="id_1" xmlns="urn:xmpp:mam:2">
                <x xmlns="jabber:x:data" type="submit">
                    <field type="hidden" var="FORM_TYPE">
                        <value>urn:xmpp:mam:2</value>
                    </field>
                    <field var="with"><value>b@prose.org</value></field>
                    <field var="before-id">
                        <value>09af3-cc343-b409f</value>
                    </field>
                </x>
                <flip-page/>
            </query>
        </iq>"#
            .to_xml_result_string()
    );

    Ok(())
}
