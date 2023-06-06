use crate::helpers::StanzaCow;
use crate::stanza::{Message, Namespace};
use crate::stanza_base;

use super::Delay;

pub struct ForwardedMessage<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> ForwardedMessage<'a> {
    pub fn new() -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("forwarded").unwrap();
        stanza.set_ns(Namespace::Forward.to_string()).unwrap();

        ForwardedMessage {
            stanza: stanza.into(),
        }
    }

    pub fn delay(&self) -> Option<Delay> {
        self.stanza
            .get_child_by_name_and_ns("delay", Namespace::Delay.to_string())
            .map(|c| c.into())
    }

    pub fn set_delay(self, delay: Delay) -> Self {
        self.add_child(delay)
    }

    pub fn message<'b>(&'a self) -> Option<Message<'b>>
    where
        'a: 'b,
    {
        self.stanza.get_child_by_name("message").map(|c| c.into())
    }

    pub fn set_message(self, message: Message) -> Self {
        self.add_child(message)
    }
}

stanza_base!(ForwardedMessage);
