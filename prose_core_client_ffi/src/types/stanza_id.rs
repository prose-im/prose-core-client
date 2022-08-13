// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::error::{Error, StanzaParseError};
use crate::types::namespace::Namespace;
use jid::BareJid;
use libstrophe::Stanza;
use std::str::FromStr;

// https://xmpp.org/extensions/xep-0359.html

#[derive(Debug, PartialEq)]
pub struct StanzaID {
    pub id: String,

    /// The value of the 'by' attribute MUST be the XMPP address of the entity assigning the unique
    /// and stable stanza ID. For one-on-one messages the assigning entity is the account. In
    /// groupchats the assigning entity is the room. Note that XMPP addresses are normalized as
    /// defined in RFC 6122.
    pub by: BareJid,
}

impl StanzaID {
    pub fn new(id: impl Into<String>, by: BareJid) -> Self {
        StanzaID { id: id.into(), by }
    }
}

impl TryFrom<&Stanza> for StanzaID {
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        Ok(StanzaID::new(
            stanza.get_attribute("id").ok_or(Error::StanzaParseError {
                error: StanzaParseError::missing_attribute("id", stanza),
            })?,
            stanza
                .get_attribute("by")
                .ok_or(StanzaParseError::missing_attribute("by", stanza))
                .and_then(|s| BareJid::from_str(s).map_err(Into::into))?,
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct OriginID {
    pub id: String,
}

impl OriginID {
    pub fn new(id: impl Into<String>) -> Self {
        OriginID { id: id.into() }
    }
}

impl TryFrom<&OriginID> for Stanza {
    type Error = Error;

    fn try_from(value: &OriginID) -> Result<Self, Self::Error> {
        let mut stanza = Stanza::new();
        stanza.set_name("origin-id")?;
        stanza.set_ns(Namespace::StanzaID)?;
        stanza.set_id(&value.id)?;
        Ok(stanza)
    }
}

#[cfg(test)]
mod tests {
    use libstrophe::Stanza;

    use super::*;

    #[test]
    fn test_deserialize_stanza_id() {
        let id = r#"<stanza-id id="Zwbe2mBjOgVQfu2A" by="marc@prose.org" xmlns="urn:xmpp:sid:0"/>"#;

        let stanza = Stanza::from_str(id);
        let result = StanzaID::try_from(&stanza).unwrap();

        assert_eq!(
            result,
            StanzaID::new(
                "Zwbe2mBjOgVQfu2A",
                BareJid::from_str("marc@prose.org").unwrap()
            )
        )
    }

    #[test]
    fn test_serialize_origin_id() {
        let id = OriginID::new("de305d54-75b4-431b-adb2-eb6b9e546013");
        let stanza = Stanza::try_from(&id).unwrap();

        assert_eq!(
            stanza.to_text().unwrap(),
            r#"<origin-id id="de305d54-75b4-431b-adb2-eb6b9e546013" xmlns="urn:xmpp:sid:0"/>"#
        );
    }
}
