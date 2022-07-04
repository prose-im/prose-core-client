use jid::BareJid;
use prose_core_client_ffi::test_helpers::mocks::{HandlerBucketExt, MockIDProvider};
use prose_core_client_ffi::{
    test_helpers::mocks::{HandlerBucket, MockConnection, StanzaBucket},
    Account, AccountObserverMock, ConnectionEvent, Result, Roster, RosterGroup, RosterItem,
    RosterItemSubscription,
};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

#[test]
fn test_loads_roster() -> Result<()> {
    let handlers = HandlerBucket::new();
    let stanzas = StanzaBucket::new();
    let observer = Arc::new(Mutex::new(AccountObserverMock::new()));

    let account = Account::new(
        MockConnection::new(handlers.clone(), stanzas.clone()),
        MockIDProvider::new(),
        Box::new(observer.clone()),
    )?;

    observer
        .lock()
        .unwrap()
        .expect_did_connect()
        .times(1)
        .returns(());

    handlers.send_connection_event(ConnectionEvent::Connect);

    stanzas.clear();

    account.roster.load_roster()?;

    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        r#"<iq id="id_1" type="get"><query xmlns="jabber:iq:roster"/></iq>"#
            .to_string()
            .trim()
    );

    let expected_roster = Roster {
        groups: vec![RosterGroup {
            name: "_default_group_".to_string(),
            items: vec![RosterItem {
                jid: BareJid::from_str("a@prose.org").unwrap(),
                subscription: RosterItemSubscription::None,
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

    // <iq id='bv1bs71f'
    // to='juliet@example.com/chamber'
    // type='result'>
    //     <query xmlns='jabber:iq:roster' ver='ver7'>
    //     <item jid='nurse@example.com'/>
    //     <item jid='romeo@example.net'/>
    //     </query>
    //     </iq>

    // let mut message_sender = MessageSenderMock::new();
    // message_sender
    //     .expect_send_message(|arg| arg.partial_eq("Paul"), |arg| arg.any())
    //     .times(..)
    //     .returns(());

    Ok(())
}
