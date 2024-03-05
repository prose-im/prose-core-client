// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use anyhow::anyhow;
use jid::BareJid;
use minidom::{Element, IntoAttributeValue};
use xmpp_parsers::message::MessagePayload;

use crate::{ns, ElementExt};

/// XEP-0372: References
/// https://xmpp.org/extensions/xep-0372.html
#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    pub r#type: ReferenceType,
    pub uri: String,
    pub anchor: Option<String>,
    pub begin: Option<usize>,
    pub end: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceType {
    Data,
    Mention,
}

impl Reference {
    pub fn data_reference(uri: impl Into<String>) -> Self {
        Self {
            r#type: ReferenceType::Data,
            uri: uri.into(),
            anchor: None,
            begin: None,
            end: None,
        }
    }

    pub fn mention(jid: BareJid) -> Self {
        Self {
            r#type: ReferenceType::Mention,
            uri: format!("xmpp:{}", jid.to_string()),
            anchor: None,
            begin: None,
            end: None,
        }
    }
}

impl TryFrom<Element> for Reference {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("reference", ns::REFERENCE)?;

        Ok(Self {
            r#type: value.attr_req("type")?.parse()?,
            uri: value.attr_req("uri")?.to_string(),
            anchor: value.attr("anchor").map(ToString::to_string),
            begin: value.attr("begin").map(|begin| begin.parse()).transpose()?,
            end: value.attr("end").map(|end| end.parse()).transpose()?,
        })
    }
}

impl From<Reference> for Element {
    fn from(value: Reference) -> Self {
        Element::builder("reference", ns::REFERENCE)
            .attr("type", value.r#type)
            .attr("uri", value.uri)
            .attr("anchor", value.anchor)
            .attr("begin", value.begin)
            .attr("end", value.end)
            .build()
    }
}

impl IntoAttributeValue for ReferenceType {
    fn into_attribute_value(self) -> Option<String> {
        match self {
            ReferenceType::Data => Some("data".to_string()),
            ReferenceType::Mention => Some("mention".to_string()),
        }
    }
}

impl FromStr for ReferenceType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "data" => Ok(Self::Data),
            "mention" => Ok(Self::Mention),
            _ => Err(anyhow!("Encountered unknown ReferenceType '{s}'")),
        }
    }
}

impl MessagePayload for Reference {}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test_deserialize_reference() -> Result<()> {
        assert_eq!(
          Reference::try_from(Element::from_str(r#"<reference 
            xmlns='urn:xmpp:reference:0'
            type='data'
            anchor='xmpp:balcony@channels.shakespeare.lit?;node=messages;item=bnhob'
            begin='72'
            end='78'
            uri='xmpp:fdp.shakespeare.lit?;node=fdp/submitted/stan.isode.net/accidentreport;item=ndina872be'
          />"#)?)?, 
          Reference {
            r#type: ReferenceType::Data,
            uri: "xmpp:fdp.shakespeare.lit?;node=fdp/submitted/stan.isode.net/accidentreport;item=ndina872be".to_string(),
            anchor: Some("xmpp:balcony@channels.shakespeare.lit?;node=messages;item=bnhob".to_string()),
            begin: Some(72),
            end: Some(78),
          });

        assert_eq!(
          Reference::try_from(Element::from_str(r#"<reference xmlns='urn:xmpp:reference:0'
              type='data'
              uri='xmpp:fdp.shakespeare.lit?;node=fdp/submitted/stan.isode.net/accidentreport;item=ndina872be'
             />"#)?)?,
          Reference {
            r#type: ReferenceType::Data,
            uri: "xmpp:fdp.shakespeare.lit?;node=fdp/submitted/stan.isode.net/accidentreport;item=ndina872be".to_string(),
            anchor: None,
            begin: None,
            end: None,
          });

        Ok(())
    }

    #[test]
    fn test_serialize_reference() -> Result<()> {
        let reference = Reference {
          r#type: ReferenceType::Data,
          uri: "xmpp:fdp.shakespeare.lit?;node=fdp/submitted/stan.isode.net/accidentreport;item=ndina872be".to_string(),
          anchor: Some("xmpp:balcony@channels.shakespeare.lit?;node=messages;item=bnhob".to_string()),
          begin: Some(72),
          end: Some(78),
        };

        assert_eq!(
            Reference::try_from(Element::from(reference.clone()))?,
            reference
        );
        Ok(())
    }
}
