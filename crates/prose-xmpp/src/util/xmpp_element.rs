// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
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

impl XMPPElement {
    pub fn try_from_element(value: Element) -> Result<Option<Self>> {
        match &value {
            _ if value.is("iq", ns::JABBER_CLIENT) => Ok(Some(Self::IQ(Iq::try_from(value)?))),
            _ if value.is("message", ns::JABBER_CLIENT) => {
                let message = xmpp_parsers::message::Message::try_from(value)?;

                if message.type_ != MessageType::Headline {
                    return Ok(Some(Self::Message(message.try_into()?)));
                }

                Ok(Some(Self::PubSubMessage(message.try_into()?)))
            }
            _ if value.is("presence", ns::JABBER_CLIENT) => {
                Ok(Some(Self::Presence(value.try_into()?)))
            }
            // Ignore certain protocol features which we might receive when running in the web app…
            _ if value.has_ns(ns::WEBSOCKET)
                | value.has_ns(ns::SASL)
                | value.has_ns(ns::STREAM) =>
            {
                Ok(None)
            }
            _ => Err(anyhow::format_err!(
                "Encountered unknown element: {}",
                String::from(&value)
            )),
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
