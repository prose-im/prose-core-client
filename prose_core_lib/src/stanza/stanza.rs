use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::stanza::Namespace;

pub struct Stanza<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Stanza<'a> {
    pub fn new(name: impl AsRef<str>) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name(name).expect("Failed to set name");
        Stanza {
            stanza: stanza.into(),
        }
    }

    pub fn new_query(ns: Namespace, query_id: Option<&str>) -> Self {
        Stanza::new("query")
            .set_namespace(ns)
            .set_attribute::<&str>("queryid", query_id)
    }

    pub fn new_text_node(name: impl AsRef<str>, text: impl AsRef<str>) -> Self {
        let mut text_node = libstrophe::Stanza::new();
        text_node.set_text(text).expect("Failed to set text");

        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name(name).expect("Failed to set name");
        stanza
            .add_child(text_node)
            .expect("Failed to add text node");

        Stanza {
            stanza: stanza.into(),
        }
    }
}

stanza_base!(Stanza);

#[cfg(test)]
mod tests {
    use crate::stanza::namespace::Namespace;

    use super::*;

    #[test]
    fn test_build_stanza() {
        let stanza = Stanza::new("iq").set_attribute("type", "set").add_child(
            Stanza::new("pubsub")
                .set_namespace(Namespace::PubSub)
                .add_child(
                    Stanza::new("subscribe")
                        .set_attribute("node", Namespace::AvatarMetadata.to_string())
                        .set_attribute("jid", "hello@prose.org"),
                ),
        );

        assert_eq!(
            stanza.to_string(),
            r#"<iq type="set"><pubsub xmlns="http://jabber.org/protocol/pubsub"><subscribe jid="hello@prose.org" node="urn:xmpp:avatar:metadata"/></pubsub></iq>"#
        );
    }
}
