use crate::helpers::StanzaCow;
use crate::stanza_base;

use crate::stanza::{message, Namespace};
use strum_macros::{Display, EnumString};

// https://xmpp.org/extensions/xep-0333.html

#[derive(Debug, PartialEq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Kind {
    Received,
    Displayed,
    Acknowledged,
}

pub struct ChatMarker<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> ChatMarker<'a> {
    pub fn new(kind: Kind, id: &message::Id) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza
            .set_name(kind.to_string())
            .expect("Failed to set name");
        stanza
            .set_ns(Namespace::ChatMarkers.to_string())
            .expect("Failed to set namespace");
        stanza
            .set_attribute("id", id.as_ref())
            .expect("Failed to set id");

        ChatMarker {
            stanza: stanza.into(),
        }
    }

    pub fn id(&self) -> Option<message::Id> {
        self.attribute("id").map(Into::into)
    }

    pub fn kind(&self) -> Option<Kind> {
        self.name().and_then(|s| s.parse::<Kind>().ok())
    }
}

stanza_base!(ChatMarker);
