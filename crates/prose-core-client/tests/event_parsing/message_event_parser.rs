// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use pretty_assertions::assert_eq;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::message::MessageType;

use prose_core_client::app::event_handlers::{MessageEvent, MessageEventType, ServerEvent};
use prose_core_client::test::parse_xml_with_current_user;
use prose_proc_macros::mt_test;
use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::message::stanza_id::StanzaId;
use prose_xmpp::stanza::message::Forwarded;
use prose_xmpp::stanza::Message;
use prose_xmpp::{bare, full};

#[mt_test]
async fn test_carbon() -> Result<()> {
    let events = parse_xml_with_current_user(
        r#"
        <message xmlns='jabber:client' type='chat' to='user@prose.org/res2' from='user@prose.org'>
            <sent xmlns='urn:xmpp:carbons:2'>
                <forwarded xmlns='urn:xmpp:forward:0'>
                    <message type='chat' xml:lang='en' from='user@prose.org/res1' id='message-id' to='other-user@prose.org' xmlns='jabber:client'>
                        <active xmlns='http://jabber.org/protocol/chatstates' />
                        <body>Hello World</body>
                        <stanza-id id='Qiuahv1eo3C222uKhOqjPiW0' by='user@prose.org' xmlns='urn:xmpp:sid:0' />
                    </message>
                </forwarded>
            </sent>
        </message>
        "#,
        full!("user@prose.org/res2")
    )
    .await?;

    assert_eq!(
        vec![ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Sync(Carbon::Sent(Forwarded {
                delay: None,
                stanza: Some(Box::new(
                    Message::new()
                        .set_id("message-id".into())
                        .set_type(MessageType::Chat)
                        .set_from(full!("user@prose.org/res1"))
                        .set_to(bare!("other-user@prose.org"))
                        .set_body("Hello World")
                        .set_chat_state(Some(ChatState::Active))
                        .set_stanza_id(StanzaId {
                            id: "Qiuahv1eo3C222uKhOqjPiW0".into(),
                            by: bare!("user@prose.org").into(),
                        })
                ))
            }))
        })],
        events
    );

    Ok(())
}
