// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::JidParseError;
use jid::{BareJid, DomainPart, NodePart};

#[derive(uniffi::Record, Debug, Clone, PartialEq)]
pub struct JID {
    pub node: Option<String>,
    pub domain: String,
}

impl JID {
    pub fn to_bare(&self) -> Result<BareJid, JidParseError> {
        Ok(BareJid::from_parts(
            self.node
                .as_ref()
                .map(|node| NodePart::new(node))
                .transpose()?
                .as_deref(),
            &DomainPart::new(&self.domain)?,
        ))
    }
}

#[uniffi::export]
pub fn parse_jid(jid: String) -> Result<JID, JidParseError> {
    Ok(jid.parse::<BareJid>()?.into())
}

impl From<BareJid> for JID {
    fn from(value: BareJid) -> Self {
        JID {
            node: value.node().map(|s| s.to_string()),
            domain: value.domain().to_string(),
        }
    }
}

impl From<JID> for BareJid {
    fn from(value: JID) -> Self {
        BareJid::from_parts(
            value
                .node
                .as_ref()
                .map(|node| NodePart::new(node).unwrap())
                .as_deref(),
            &DomainPart::new(&value.domain).unwrap(),
        )
    }
}

impl ToString for JID {
    fn to_string(&self) -> String {
        BareJid::from(self.clone()).into_inner()
    }
}

#[uniffi::export]
pub fn format_jid(jid: JID) -> String {
    jid.to_string()
}
