// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::ns;
use jid::Jid;
use minidom::Element;
use xmpp_parsers::message::Message;
use xmpp_parsers::pubsub::PubSubEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct PubSubMessage {
    pub from: Jid,
    pub events: Vec<PubSubEvent>,
}

impl TryFrom<Message> for PubSubMessage {
    type Error = anyhow::Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let Some(from) = value.from else {
            return Err(anyhow::format_err!("Missing from in PubSub message"));
        };

        Ok(PubSubMessage {
            from,
            events: value
                .payloads
                .into_iter()
                .filter_map(|child| {
                    if !child.is("event", ns::PUBSUB_EVENT) {
                        return None;
                    }
                    Some(PubSubEvent::try_from(child))
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<PubSubMessage> for Element {
    fn from(value: PubSubMessage) -> Self {
        Element::builder("message", ns::JABBER_CLIENT)
            .attr("from", value.from)
            .append_all(value.events)
            .build()
    }
}
