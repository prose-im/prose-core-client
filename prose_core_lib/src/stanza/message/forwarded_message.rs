use crate::helpers::StanzaCow;
use crate::stanza::{Message, Namespace};
use crate::stanza_base;

use super::Delay;

pub struct ForwardedMessage<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> ForwardedMessage<'a> {
    pub fn delay(&self) -> Option<Delay> {
        self.stanza
            .get_child_by_name_and_ns("delay", Namespace::Delay.to_string())
            .map(|c| c.into())
    }

    pub fn message<'b>(&'a self) -> Option<Message<'b>>
    where
        'a: 'b,
    {
        self.stanza.get_child_by_name("message").map(|c| c.into())
    }
}

stanza_base!(ForwardedMessage);
