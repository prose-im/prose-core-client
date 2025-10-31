// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
use xmpp_parsers::delay::Delay;

use crate::stanza::message::Message;

#[derive(Debug, PartialEq, Clone)]
pub struct Forwarded {
    pub delay: Option<Delay>,
    pub message: Box<Message>,
}

impl TryFrom<Element> for Forwarded {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        xmpp_parsers::forwarding::Forwarded::try_from(value)?.try_into()
    }
}

impl TryFrom<xmpp_parsers::forwarding::Forwarded> for Forwarded {
    type Error = anyhow::Error;

    fn try_from(value: xmpp_parsers::forwarding::Forwarded) -> Result<Self, Self::Error> {
        let message = Message::try_from(value.message)?;
        Ok(Forwarded {
            delay: value.delay,
            message: Box::new(message),
        })
    }
}

impl From<Forwarded> for Element {
    fn from(value: Forwarded) -> Self {
        xmpp_parsers::forwarding::Forwarded::from(value).into()
    }
}

impl From<Forwarded> for xmpp_parsers::forwarding::Forwarded {
    fn from(value: Forwarded) -> Self {
        xmpp_parsers::forwarding::Forwarded {
            delay: value.delay,
            message: (*value.message).into(),
        }
    }
}

impl From<Box<Message>> for Element {
    fn from(value: Box<Message>) -> Self {
        value.into()
    }
}
