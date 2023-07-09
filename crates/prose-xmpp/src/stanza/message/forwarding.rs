use minidom::Element;
use xmpp_parsers::delay::Delay;

use crate::stanza::message::Message;

#[derive(Debug, PartialEq, Clone)]
pub struct Forwarded {
    pub delay: Option<Delay>,
    pub stanza: Option<Box<Message>>,
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
        Ok(Forwarded {
            delay: value.delay,
            stanza: value
                .stanza
                .map(TryInto::try_into)
                .transpose()?
                .map(Box::new),
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
            stanza: value.stanza.map(|s| *s).map(Into::into),
        }
    }
}

impl From<Box<Message>> for Element {
    fn from(value: Box<Message>) -> Self {
        value.into()
    }
}
