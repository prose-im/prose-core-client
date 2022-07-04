use prose_core_client_ffi::test_helpers::mocks::{HandlerBucketExt, MockIDProvider};
use prose_core_client_ffi::{
    test_helpers::mocks::{HandlerBucket, MockAccountObserver, MockConnection, StanzaBucket},
    Account, ConnectionEvent, Result,
};

#[test]
fn test_sends_empty_presence_on_connect() -> Result<()> {
    let handlers = HandlerBucket::new();
    let stanzas = StanzaBucket::new();
    let _account = Account::new(
        MockConnection::new(handlers.clone(), stanzas.clone()),
        MockIDProvider::new(),
        MockAccountObserver::new(),
    );

    handlers.send_connection_event(ConnectionEvent::Connect);

    assert_eq!(stanzas.stanzas.borrow().len(), 1);
    assert_eq!(
        stanzas.stanzas.borrow().first().unwrap().to_text()?,
        "<presence/>".to_string()
    );

    Ok(())
}
