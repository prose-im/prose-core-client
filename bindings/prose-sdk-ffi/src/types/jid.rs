use jid::{BareJid, DomainPart, Error as JidParseError, FullJid, NodePart, ResourcePart};

#[derive(Debug, Clone, PartialEq)]
pub struct JID {
    pub node: Option<String>,
    pub domain: String,
}

impl JID {
    pub fn to_full_jid_with_resource(&self, resource: &str) -> Result<FullJid, JidParseError> {
        Ok(FullJid::from_parts(
            self.node
                .as_ref()
                .map(|node| NodePart::new(node))
                .transpose()?
                .as_ref(),
            &DomainPart::new(&self.domain)?,
            &ResourcePart::new(resource)?,
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
                .as_ref(),
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
