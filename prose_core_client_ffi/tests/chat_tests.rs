use jid::BareJid;
use prose_core_client_ffi::test_helpers::StrExt;
use prose_core_client_ffi::{Account, Result};
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
