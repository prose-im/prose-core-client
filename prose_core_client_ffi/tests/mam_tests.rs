use jid::BareJid;
use prose_core_client_ffi::test_helpers::mocks::HandlerBucketExt;
use prose_core_client_ffi::{Account, Result, XMPPMAMDefaultBehavior, XMPPMAMPreferences};
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
