// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
use xmpp_parsers::iq::Iq;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::presence::Presence;

use crate::ns;
use crate::stanza::{Message, PubSubMessage};

#[derive(Debug, Clone)]
pub enum XMPPElement {
    Presence(Presence),
    Message(Message),
    IQ(Iq),
    PubSubMessage(PubSubMessage),
}

impl TryFrom<Element> for XMPPElement {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        if value.is("iq", ns::JABBER_CLIENT) {
            Ok(Self::IQ(Iq::try_from(value)?))
        } else if value.is("message", ns::JABBER_CLIENT) {
            let message = xmpp_parsers::message::Message::try_from(value)?;

            if message.type_ != MessageType::Headline {
                return Ok(Self::Message(message.try_into()?));
            }

            Ok(Self::PubSubMessage(message.try_into()?))
        } else if value.is("presence", ns::JABBER_CLIENT) {
            Ok(Self::Presence(value.try_into()?))
        } else {
            Err(anyhow::format_err!("Encountered unknown element"))
        }
    }
}

impl From<XMPPElement> for Element {
    fn from(value: XMPPElement) -> Self {
        match value {
            XMPPElement::Presence(stanza) => stanza.into(),
            XMPPElement::Message(stanza) => stanza.into(),
            XMPPElement::IQ(stanza) => stanza.into(),
            XMPPElement::PubSubMessage(stanza) => stanza.into(),
        }
    }
}

impl From<XMPPElement> for String {
    fn from(elem: XMPPElement) -> String {
        String::from(&Element::from(elem))
    }
}
