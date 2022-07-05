use jid::BareJid;
use prose_core_client_ffi::test_helpers::mocks::HandlerBucketExt;
use prose_core_client_ffi::{
    Account, Result, XMPPPresence, XMPPPresenceKind, XMPPRoster, XMPPRosterGroup, XMPPRosterItem,
    XMPPRosterItemSubscription,
};
use std::str::FromStr;

#[test]
fn test_loads_roster() -> Result<()> {
    let (account, handlers, stanzas, observer) = Account::connected();

    account.roster.load_roster()?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"<iq id="id_1" type="get"><query xmlns="jabber:iq:roster"/></iq>"#.to_string()
    );

    let expected_roster = XMPPRoster {
        groups: vec![XMPPRosterGroup {
            name: "_default_group_".to_string(),
            items: vec![XMPPRosterItem {
                jid: BareJid::from_str("a@prose.org").unwrap(),
                subscription: XMPPRosterItemSubscription::None,
            }],
        }],
    };

    observer
        .lock()
        .unwrap()
        .expect_did_receive_roster(|arg| arg.partial_eq(expected_roster))
        .times(1)
        .returns(());

    handlers.send_stanza_str(
        r#"
  <iq type="result">
    <query xmlns='jabber:iq:roster' ver='ver7'><item jid="a@prose.org"/></query>
  </iq>
  "#,
    );

    Ok(())
}

#[test]
fn test_adds_user() -> Result<()> {
    let (account, _, stanzas, _) = Account::connected();

    account.roster.add_user(
        &BareJid::from_str("a@prose.org").unwrap(),
        Some("Nickname"),
        &["Group 1", "Group 2"],
    )?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"<iq id="id_1" type="set"><query xmlns="jabber:iq:roster"><item jid="a@prose.org" name="Nickname"><group>Group 1</group><group>Group 2</group></item></query></iq>"#
            .to_string()
    );

    Ok(())
}

#[test]
fn test_removes_user_and_unsubscribes_from_presence() -> Result<()> {
    let (account, _, stanzas, _) = Account::connected();

    account
        .roster
        .remove_user_and_unsubscribe_from_presence(&BareJid::from_str("a@prose.org").unwrap())?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"<iq id="id_1" type="set"><query xmlns="jabber:iq:roster"><item jid="a@prose.org" subscription="remove"/></query></iq>"#
            .to_string()
    );

    Ok(())
}

#[test]
fn test_subscribes_to_user_presence() -> Result<()> {
    let (account, _, stanzas, _) = Account::connected();

    account
        .roster
        .subscribe_to_user_presence(&BareJid::from_str("a@prose.org").unwrap())?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"<presence id="id_1" type="subscribe" to="a@prose.org"/>"#.to_string()
    );

    Ok(())
}

#[test]
fn test_unsubscribes_from_user_presence() -> Result<()> {
    let (account, _, stanzas, _) = Account::connected();

    account
        .roster
        .unsubscribe_from_user_presence(&BareJid::from_str("a@prose.org").unwrap())?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"<presence id="id_1" type="unsubscribe" to="a@prose.org"/>"#.to_string()
    );

    Ok(())
}

#[test]
fn test_grants_presence_submission() -> Result<()> {
    let (account, _, stanzas, _) = Account::connected();

    account
        .roster
        .grant_presence_permission_to_user(&BareJid::from_str("a@prose.org").unwrap())?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"<presence id="id_1" type="subscribed" to="a@prose.org"/>"#.to_string()
    );

    Ok(())
}

#[test]
fn test_revokes_presence_submission() -> Result<()> {
    let (account, _, stanzas, _) = Account::connected();

    account
        .roster
        .revoke_or_reject_presence_permission_from_user(
            &BareJid::from_str("a@prose.org").unwrap(),
        )?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"<presence id="id_1" type="unsubscribed" to="a@prose.org"/>"#.to_string()
    );

    Ok(())
}

#[test]
fn test_receives_subscription_request() -> Result<()> {
    let (_, handlers, _, observer) = Account::connected();

    observer
        .lock()
        .unwrap()
        .expect_did_receive_presence(|arg| {
            arg.partial_eq(XMPPPresence::new(
                Some(XMPPPresenceKind::Subscribe),
                Some(BareJid::from_str("a@prose.org").unwrap()),
                None,
                None,
                None,
            ))
        })
        .times(1)
        .returns(());

    observer
        .lock()
        .unwrap()
        .expect_did_receive_presence_subscription_request(|arg| {
            arg.partial_eq(BareJid::from_str("a@prose.org").unwrap())
        })
        .times(1)
        .returns(());

    handlers.send_stanza_str(r#"<presence type="subscribe" from="a@prose.org"/>"#);

    Ok(())
}
