use crate::ns;
use jid::Jid;
use xmpp_parsers::message::Message;
use xmpp_parsers::pubsub::PubSubEvent;

#[derive(Debug, Clone)]
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