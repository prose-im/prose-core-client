use crate::error::{Error, StanzaParseError};
use chrono::DateTime;
use jid::BareJid;
use libstrophe::Stanza;
use std::str::FromStr;

// https://xmpp.org/extensions/xep-0203.html

#[derive(Debug, PartialEq)]
pub struct Delay {
    /// The time when the XML stanza was originally sent. The format MUST adhere to the dateTime
    /// format specified in XEP-0082 and MUST be expressed in UTC.
    pub stamp: i64,

    /// The Jabber ID of the entity that originally sent the XML stanza or that delayed the
    /// delivery of the stanza (e.g., the address of a multi-user chat room).
    pub from: Option<BareJid>,
}

impl Delay {
    pub fn new(stamp: i64, from: Option<BareJid>) -> Self {
        Delay { stamp, from }
    }
}

impl TryFrom<&Stanza> for Delay {
    type Error = Error;

    fn try_from(stanza: &Stanza) -> std::result::Result<Self, Self::Error> {
        Ok(Delay::new(
            stanza
                .get_attribute("stamp")
                .ok_or(Error::StanzaParseError {
                    error: StanzaParseError::missing_attribute("stamp", stanza),
                })
                .and_then(|s| DateTime::parse_from_rfc3339(s).map_err(Into::into))
                .map(|t| t.timestamp())?,
            stanza
                .get_attribute("from")
                .and_then(|s| BareJid::from_str(s).ok()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libstrophe::Stanza;

    #[test]
    fn test_deserialize_without_from() {
        let message = r#"<delay xmlns="urn:xmpp:delay" stamp="2002-09-10T23:08:25Z"/>"#;

        let stanza = Stanza::from_str(message);
        let delay = Delay::try_from(&stanza).unwrap();

        assert_eq!(delay, Delay::new(1031699305, None));
    }

    #[test]
    fn test_deserialize_with_non_utc_timezone() {
        let message = r#"<delay xmlns="urn:xmpp:delay" stamp="2002-09-10T23:08:25-08:00"/>"#;

        let stanza = Stanza::from_str(message);
        let delay = Delay::try_from(&stanza).unwrap();

        assert_eq!(delay, Delay::new(1031728105, None));
    }

    #[test]
    fn test_deserialize() {
        let message =
            r#"<delay xmlns="urn:xmpp:delay" from="d@prose.org" stamp="2002-09-10T23:08:25Z"/>"#;

        let stanza = Stanza::from_str(message);
        let delay = Delay::try_from(&stanza).unwrap();

        assert_eq!(
            delay,
            Delay::new(1031699305, Some(BareJid::from_str("d@prose.org").unwrap()))
        );
    }
}
