use crate::helpers::StanzaCow;
use crate::stanza::{message, Namespace};
use crate::stanza_base;

/// XEP-0422
pub struct MessageFastening<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> MessageFastening<'a> {
    pub fn new(id: impl Into<message::Id>, retract: bool) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("apply-to").expect("Failed to set name");
        stanza
            .set_ns(Namespace::Fasten.to_string())
            .expect("Failed to set namespace");
        stanza
            .set_attribute("id", id.into())
            .expect("Failed to set attribute");

        if retract {
            let mut retract = libstrophe::Stanza::new();
            retract.set_name("retract").expect("Failed to set name");
            retract
                .set_ns(Namespace::Retract.to_string())
                .expect("Failed to set namespace");
            stanza.add_child(retract).expect("Failed to add child");
        }

        MessageFastening {
            stanza: stanza.into(),
        }
    }

    pub fn id(&self) -> Option<message::Id> {
        self.attribute("id").map(Into::into)
    }

    pub fn retract(&self) -> bool {
        self.child_by_name_and_namespace("retract", Namespace::Retract)
            .is_some()
    }
}

impl<'a> MessageFastening<'a> {}

stanza_base!(MessageFastening);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_deserialize_empty_fastening() {
        let fastening = r#"
        <apply-to xmlns="urn:xmpp:fasten:0" id="origin-id-1"/>
        "#;

        let fastening = MessageFastening::from_str(fastening).unwrap();

        assert_eq!(fastening.id(), Some("origin-id-1".into()));
        assert_eq!(fastening.retract(), false);
    }

    #[test]
    fn test_deserialize_retraction() {
        let fastening = r#"
        <apply-to xmlns="urn:xmpp:fasten:0" id="message-id">
          <retract xmlns='urn:xmpp:message-retract:0'/>
        </apply-to>
        "#;

        let fastening = MessageFastening::from_str(fastening).unwrap();

        assert_eq!(fastening.id(), Some("message-id".into()));
        assert_eq!(fastening.retract(), true);
    }
}
