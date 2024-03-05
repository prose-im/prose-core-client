// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use jid::Jid;

pub trait JidExt {
    fn from_iri(iri: &str) -> Result<Jid, JidParseError>;
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum JidParseError {
    #[error("Missing xmpp: prefix in IRI")]
    InvalidIRI,
    #[error(transparent)]
    JID(#[from] jid::Error),
}

impl JidExt for Jid {
    fn from_iri(iri: &str) -> Result<Self, JidParseError> {
        let Some(mut iri) = iri.strip_prefix("xmpp:") else {
            return Err(JidParseError::InvalidIRI);
        };
        if let Some(idx) = iri.rfind("?join") {
            iri = &iri[..idx];
        }
        Ok(Self::from_str(iri)?)
    }
}

#[cfg(test)]
mod tests {
    use prose_xmpp::jid;

    use super::*;

    #[test]
    fn test_from_iri() {
        assert!(Jid::from_iri("").is_err());
        assert_eq!(
            Jid::from_iri("xmpp:room@muc.example.org?join"),
            Ok(jid!("room@muc.example.org"))
        );
        assert_eq!(
            Jid::from_iri("xmpp:room@muc.example.org"),
            Ok(jid!("room@muc.example.org"))
        );
    }
}
