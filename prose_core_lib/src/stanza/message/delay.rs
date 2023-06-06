use crate::helpers::StanzaCow;
use crate::stanza::Namespace;
use crate::stanza_base;
use chrono::{DateTime, Utc};

// https://xmpp.org/extensions/xep-0203.html

pub struct Delay<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Delay<'a> {
    pub fn new(stamp: DateTime<Utc>) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("delay").unwrap();
        stanza.set_ns(Namespace::Delay.to_string()).unwrap();
        stanza.set_attribute("stamp", stamp.to_rfc3339()).unwrap();

        Delay {
            stanza: stanza.into(),
        }
    }

    /// The time when the XML stanza was originally sent. The format MUST adhere to the dateTime
    /// format specified in XEP-0082 and MUST be expressed in UTC.
    pub fn stamp(&self) -> Option<DateTime<Utc>> {
        self.attribute("stamp")
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|t| t.with_timezone(&Utc))
    }
}

stanza_base!(Delay);

#[cfg(test)]
mod tests {
    use super::*;
    use jid::BareJid;

    #[test]
    fn test_deserialize_without_from() {
        let message = r#"<delay xmlns="urn:xmpp:delay" stamp="2002-09-10T23:08:25Z"/>"#;
        let delay = Delay::from_str(message).unwrap();
        assert_eq!(delay.stamp().map(|t| t.timestamp()), Some(1031699305));
    }

    #[test]
    fn test_deserialize_with_non_utc_timezone() {
        let message = r#"<delay xmlns="urn:xmpp:delay" stamp="2002-09-10T23:08:25-08:00"/>"#;
        let delay = Delay::from_str(message).unwrap();
        assert_eq!(delay.stamp().map(|t| t.timestamp()), Some(1031728105));
    }

    #[test]
    fn test_deserialize() {
        let message =
            r#"<delay xmlns="urn:xmpp:delay" from="d@prose.org" stamp="2002-09-10T23:08:25Z"/>"#;
        let delay = Delay::from_str(message).unwrap();

        assert_eq!(delay.stamp().map(|t| t.timestamp()), Some(1031699305));
        assert_eq!(
            delay.from(),
            Some(BareJid::from_str("d@prose.org").unwrap().into())
        );
    }
}
