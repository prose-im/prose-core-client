use crate::helpers::id_string_macro::id_string;
use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::stanza::{Namespace, PubSub};

use super::Kind;

id_string!(Id);

pub struct IQ<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> IQ<'a> {
    pub fn new(kind: Kind, id: impl Into<Id>) -> Self {
        IQ {
            stanza: libstrophe::Stanza::new_iq(
                Some(kind.to_string().as_ref()),
                Some(id.into().as_ref()),
            )
            .into(),
        }
    }
}

impl<'a> IQ<'a> {
    pub fn kind(&self) -> Option<Kind> {
        self.stanza
            .get_attribute("type")
            .and_then(|s| s.parse::<Kind>().ok())
    }

    pub fn id(&self) -> Option<Id> {
        self.stanza.get_attribute("id").map(|s| s.into())
    }

    pub fn pubsub(&self) -> Option<PubSub> {
        self.child_by_name_and_namespace("pubsub", Namespace::PubSub)
            .map(Into::into)
    }
}

stanza_base!(IQ);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use jid::Jid;

    use super::*;

    #[test]
    fn test_builder() {
        let iq = IQ::new(Kind::Get, "my-id")
            .set_to(Jid::from_str("a@prose.org").unwrap())
            .set_from(Jid::from_str("b@prose.org").unwrap());

        assert_eq!(
            iq.to_string(),
            r#"<iq id="my-id" to="a@prose.org" type="get" from="b@prose.org"/>"#
        );
    }

    #[test]
    fn test_copy_on_write() {
        let mut stanza = libstrophe::Stanza::new_iq(Some("get"), Some("my-id"));
        stanza.set_from("a@prose.org").unwrap();

        let iq = <IQ as From<&libstrophe::Stanza>>::from(&stanza)
            .set_from(Jid::from_str("iq1@prose.org").unwrap());

        assert_eq!(
            stanza.to_string(),
            r#"<iq id="my-id" type="get" from="a@prose.org"/>"#
        );
        assert_eq!(
            iq.to_string(),
            r#"<iq id="my-id" type="get" from="iq1@prose.org"/>"#
        );
    }

    #[test]
    fn test_base_impl() {
        let iq = IQ::new(Kind::Get, "my-id")
            .set_to(Jid::from_str("a@prose.org").unwrap())
            .set_attribute("test", "value");

        assert_eq!(
            iq.to_string(),
            r#"<iq id="my-id" test="value" to="a@prose.org" type="get"/>"#
        );
    }
}
