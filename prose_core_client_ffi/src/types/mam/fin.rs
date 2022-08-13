// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::error::{Error, StanzaParseError};
use crate::types::namespace::Namespace;
use crate::MessageId;
use libstrophe::Stanza;

#[derive(Debug, PartialEq)]
pub struct Fin {
    pub query_id: Option<String>,
    pub first_message_id: Option<MessageId>,
    pub last_message_id: Option<MessageId>,
    pub complete: bool,
    pub count: Option<i64>,
}

impl Fin {
    pub fn new(
        query_id: Option<&str>,
        first_message_id: Option<MessageId>,
        last_message_id: Option<MessageId>,
        complete: bool,
        count: Option<i64>,
    ) -> Self {
        Fin {
            query_id: query_id.map(str::to_string),
            first_message_id: first_message_id.into(),
            last_message_id: last_message_id.into(),
            complete,
            count,
        }
    }
}

impl TryFrom<&Stanza> for Fin {
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        let set_node = stanza
            .get_child_by_name_and_ns("set", Namespace::RSM)
            .ok_or(Error::StanzaParseError {
                error: StanzaParseError::missing_child_node("set", stanza),
            })?;

        Ok(Fin::new(
            stanza.get_attribute("queryid"),
            set_node
                .get_child_by_name("first")
                .and_then(|n| n.text())
                .map(|t| t.into()),
            set_node
                .get_child_by_name("last")
                .and_then(|n| n.text())
                .map(|t| t.into()),
            stanza
                .get_attribute("complete")
                .and_then(|s| Some(s == "true"))
                .unwrap_or(false),
            set_node
                .get_child_by_name("count")
                .and_then(|n| n.text())
                .and_then(|s| i64::from_str_radix(&s, 10).ok()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libstrophe::Stanza;

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

        let stanza = Stanza::from_str(fin);
        let fin = Fin::try_from(&stanza).unwrap();

        assert_eq!(
            fin,
            Fin::new(
                None,
                Some("23452-4534-1".into()),
                Some("390-2342-22".into()),
                true,
                Some(16)
            )
        );
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

        let stanza = Stanza::from_str(fin);
        let fin = Fin::try_from(&stanza).unwrap();

        assert_eq!(
            fin,
            Fin::new(
                Some("my-query-id"),
                Some("23452-4534-1".into()),
                Some("390-2342-22".into()),
                false,
                None
            )
        );
    }
}
