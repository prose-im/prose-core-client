use crate::error::{Error, StanzaParseError};
use crate::types::namespace::Namespace;
use base64;
use libstrophe::Stanza;
use sha1::{Digest, Sha1};

#[derive(Debug, PartialEq)]
pub struct AvatarData {
    pub data: Vec<u8>,
    pub sha1: String,
}

impl AvatarData {
    pub fn new(data: Vec<u8>) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(&data);
        let sha1 = format!("{:x}", hasher.finalize());

        AvatarData { data, sha1 }
    }
}

impl TryFrom<&Stanza> for AvatarData {
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        if stanza.name() != Some("data") || stanza.ns() != Some(Namespace::AvatarData) {
            return Err(StanzaParseError::missing_child_node("data", stanza).into());
        }

        let base64_data = match stanza.get_first_child().and_then(|n| n.text()) {
            Some(data) => data,
            None => return Err(StanzaParseError::missing_text("data", stanza).into()),
        };

        let data =
            base64::decode(base64_data).map_err(|e| Error::StanzaParseError { error: e.into() })?;

        Ok(AvatarData::new(data))
    }
}

#[cfg(test)]
mod tests {
    use libstrophe::Stanza;

    use super::*;

    #[test]
    fn test_deserializes_avatar_data() {
        let xml = r#"
        <data xmlns="urn:xmpp:avatar:data">iVBORw0KGgoAAAANSUhEUgAAAlgAAAJYAQMAAACEqAqfAAAAA1BMVEX/AP804Oa6AAAAQ0lEQVR4Ae3BAQ0AAADCIPunfg43YAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA5wKyIAAB5pA9iQAAAABJRU5ErkJggg==</data>
        "#;

        let stanza = Stanza::from_str(xml);
        let data = AvatarData::try_from(&stanza).unwrap();

        assert_eq!(data.data.len(), 139);
        assert_eq!(data.sha1, "c1fc608fe89995e52457da8364672061af949a94");
    }
}
