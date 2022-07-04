use prose_core_client_ffi::test_helpers::mocks::{HandlerBucketExt, MockIDProvider};
use prose_core_client_ffi::{
    test_helpers::mocks::{HandlerBucket, MockConnection, StanzaBucket},
    Account, AccountObserverMock, ConnectionEvent, Result,
};

#[test]
fn test_sends_empty_presence_on_connect() -> Result<()> {
    let mut observer = AccountObserverMock::new();
    observer.expect_did_connect().times(1).returns(());

    let handlers = HandlerBucket::new();
    let stanzas = StanzaBucket::new();
    let _account = Account::new(
        MockConnection::new(handlers.clone(), stanzas.clone()),
        MockIDProvider::new(),
        Box::new(observer),
    );

    handlers.send_connection_event(ConnectionEvent::Connect);

    assert_eq!(stanzas.stanzas.borrow().len(), 1);
    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        "<presence/>".to_string()
    );

    Ok(())
}
