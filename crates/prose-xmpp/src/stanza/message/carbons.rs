// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::stanza::message::Forwarded;
use minidom::Element;

#[derive(Debug, PartialEq, Clone)]
pub struct Received {
    pub forwarded: Forwarded,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Sent {
    pub forwarded: Forwarded,
}

impl TryFrom<Element> for Received {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        let received = xmpp_parsers::carbons::Received::try_from(value)?;
        Ok(Received {
            forwarded: received.forwarded.try_into()?,
        })
    }
}

impl From<Received> for Element {
    fn from(value: Received) -> Self {
        xmpp_parsers::carbons::Received {
            forwarded: value.forwarded.into(),
        }
        .into()
    }
}

impl TryFrom<Element> for Sent {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        let sent = xmpp_parsers::carbons::Sent::try_from(value)?;
        Ok(Sent {
            forwarded: sent.forwarded.try_into()?,
        })
    }
}

impl From<Sent> for Element {
    fn from(value: Sent) -> Self {
        xmpp_parsers::carbons::Sent {
            forwarded: value.forwarded.into(),
        }
        .into()
    }
}
