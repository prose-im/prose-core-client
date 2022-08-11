use jid::BareJid;
use prose_core_client_ffi::test_helpers::mocks::HandlerBucketExt;
use prose_core_client_ffi::test_helpers::StrExt;
use prose_core_client_ffi::{Account, Result, XMPPForwardedMessage, XMPPMessage};
use std::str::FromStr;

#[test]
fn test_sends_reactions() -> Result<()> {
    let (account, _, stanzas, _) = Account::connected();

    account.chat.send_reactions(
        "my-message-id".into(),
        &BareJid::from_str("a@prose.org").unwrap(),
        &vec!["🐇", "🐰"],
    )?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"
        <message id="id_1" to="a@prose.org" type="chat">
            <reactions id="my-message-id" xmlns="urn:xmpp:reactions:0">
                <reaction>🐇</reaction>
                <reaction>🐰</reaction>
            </reactions>
        </message>
        "#
        .to_xml_result_string()
    );

    Ok(())
}

#[test]
fn test_retracts_message() -> Result<()> {
    let (account, _, stanzas, _) = Account::connected();

    account.chat.retract_message(
        "my-message-id".into(),
        &BareJid::from_str("a@prose.org").unwrap(),
    )?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"
        <message id="id_1" to="a@prose.org" type="chat">
            <apply-to id="my-message-id" xmlns="urn:xmpp:fasten:0">
                <retract xmlns="urn:xmpp:message-retract:0"/>
            </apply-to>
            <fallback xmlns="urn:xmpp:fallback:0"/>
            <body>This person attempted to retract a previous message, but it's unsupported by your client.</body>
        </message>
        "#
            .to_xml_result_string()
    );

    Ok(())
}

#[test]
fn test_receives_message_carbons() -> Result<()> {
    let (_, handlers, _, observer) = Account::connected();

    let expected_message = XMPPForwardedMessage::new(
        XMPPMessage::new_chat_message(
            BareJid::from_str("a@prose.org").unwrap(),
            BareJid::from_str("test@prose.org").unwrap(),
            None,
            "My Message",
            None,
        ),
        None,
    );

    observer
        .lock()
        .unwrap()
        .expect_did_receive_message_carbon(|arg| arg.partial_eq(expected_message))
        .times(1)
        .returns(());

    handlers.send_stanza_str(
        r#"
        <message to="test@prose.org/ci" type="chat" from="test@prose.org">
            <received xmlns="urn:xmpp:carbons:2">
                <forwarded xmlns="urn:xmpp:forward:0">
                    <message xmlns="jabber:client" to="test@prose.org/void" type="chat" from="a@prose.org/adium">
                        <body>My Message</body>
                    </message>
                </forwarded>
            </received>
        </message>
  "#,
    );

    Ok(())
}

#[test]
fn test_receives_sent_message_carbons() -> Result<()> {
    let (_, handlers, _, observer) = Account::connected();

    let expected_message = XMPPForwardedMessage::new(
        XMPPMessage::new_chat_message(
            BareJid::from_str("test@prose.org").unwrap(),
            BareJid::from_str("a@prose.org").unwrap(),
            None,
            "My Message",
            None,
        ),
        None,
    );

    observer
        .lock()
        .unwrap()
        .expect_did_receive_sent_message_carbon(|arg| arg.partial_eq(expected_message))
        .times(1)
        .returns(());

    handlers.send_stanza_str(
        r#"
        <message to="test@prose.org/ci" type="chat" from="test@prose.org">
            <sent xmlns="urn:xmpp:carbons:2">
                <forwarded xmlns="urn:xmpp:forward:0">
                    <message xmlns="jabber:client" to="a@prose.org/adium" type="chat" from="test@prose.org/void">
                        <body>My Message</body>
                    </message>
                </forwarded>
            </sent>
        </message>
  "#,
    );

    Ok(())
}
