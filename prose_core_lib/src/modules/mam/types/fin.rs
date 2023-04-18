use crate::helpers::StanzaCow;
use crate::stanza::{message, Namespace};
use crate::stanza_base;

// https://xmpp.org/extensions/xep-0203.html

pub struct Fin<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Fin<'a> {
    pub fn query_id(&self) -> Option<&str> {
        self.attribute("queryid")
    }

    pub fn first_message_id(&self) -> Option<message::StanzaId> {
        self.child_by_name_and_namespace("set", Namespace::RSM)?
            .child_by_name("first")?
            .text()
            .map(|f| f.into())
    }

    pub fn last_message_id(&self) -> Option<message::StanzaId> {
        self.child_by_name_and_namespace("set", Namespace::RSM)?
            .child_by_name("last")?
            .text()
            .map(|f| f.into())
    }

    pub fn is_complete(&self) -> bool {
        let Some(val) = self.attribute("complete") else {
            return false
        };
        val.to_lowercase() == "true"
    }

    pub fn count(&self) -> Option<i64> {
        self.child_by_name_and_namespace("set", Namespace::RSM)?
            .child_by_name("count")?
            .text()
            .and_then(|s| i64::from_str_radix(&s, 10).ok())
    }
}

stanza_base!(Fin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_fin() {
        let fin = r#"
        <fin xmlns='urn:xmpp:mam:2' complete='true'>
          <set xmlns='http://jabber.org/protocol/rsm'>
            <first index='0'>23452-4534-1</first>
            <last>390-2342-22</last>
            <count>16</count>
          </set>
        </fin>
        "#;

        let fin = Fin::from_str(&fin).unwrap();

        assert_eq!(fin.query_id(), None);
        assert_eq!(fin.first_message_id(), Some("23452-4534-1".into()));
        assert_eq!(fin.last_message_id(), Some("390-2342-22".into()));
        assert_eq!(fin.is_complete(), true);
        assert_eq!(fin.count(), Some(16));
    }

    #[test]
    fn test_deserialize_fin_without_count() {
        let fin = r#"
        <fin xmlns='urn:xmpp:mam:2' queryid='my-query-id'>
          <set xmlns='http://jabber.org/protocol/rsm'>
            <first index='0'>23452-4534-1</first>
            <last>390-2342-22</last>
          </set>
        </fin>
        "#;

        let fin = Fin::from_str(&fin).unwrap();

        assert_eq!(fin.query_id(), Some("my-query-id"));
        assert_eq!(fin.first_message_id(), Some("23452-4534-1".into()));
        assert_eq!(fin.last_message_id(), Some("390-2342-22".into()));
        assert_eq!(fin.is_complete(), false);
        assert_eq!(fin.count(), None);
    }
}
