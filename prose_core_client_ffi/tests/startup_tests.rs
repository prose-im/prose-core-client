// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::FullJid;
use prose_core_client_ffi::test_helpers::mocks::{HandlerBucketExt, MockIDProvider};
use prose_core_client_ffi::test_helpers::StrExt;
use prose_core_client_ffi::{
    test_helpers::mocks::{HandlerBucket, MockConnection, StanzaBucket},
    Account, AccountObserverMock, ConnectionEvent, Result,
};
use std::str::FromStr;

#[test]
fn test_sends_empty_presence_on_connect() -> Result<()> {
    let mut observer = AccountObserverMock::new();
    observer.expect_did_connect().times(1).returns(());

    let handlers = HandlerBucket::new();
    let stanzas = StanzaBucket::new();
    let _account = Account::new(
        &FullJid::from_str("test@prose.org/ci").unwrap(),
        MockConnection::new(handlers.clone(), stanzas.clone()),
        MockIDProvider::new(0),
        Box::new(observer),
    );

    handlers.send_connection_event(ConnectionEvent::Connect);

    assert_eq!(stanzas.stanzas.borrow().len(), 3);
    assert_eq!(stanzas.stanza_at_index(0).to_text()?, "<presence/>");
    assert_eq!(
        stanzas.stanza_at_index(1).to_text()?,
        r#"<iq id="id_1" type="set" from="test@prose.org/ci"><enable xmlns="urn:xmpp:carbons:2"/></iq>"#
    );
    assert_eq!(
        stanzas.stanza_at_index(2).to_text()?,
        r#"
        <iq type="set">
            <pubsub xmlns="http://jabber.org/protocol/pubsub">
                <subscribe jid="test@prose.org/ci" node="urn:xmpp:avatar:data"/>
            </pubsub>
        </iq>
        "#
        .to_xml_result_string()
    );

    Ok(())
}
