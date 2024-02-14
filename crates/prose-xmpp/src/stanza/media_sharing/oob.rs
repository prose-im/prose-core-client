// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::anyhow;
use minidom::Element;
use xmpp_parsers::message::MessagePayload;

use crate::{ns, ElementExt};

#[derive(Debug, PartialEq, Clone)]
pub struct OOB {
    pub url: String,
    pub desc: Option<String>,
}

impl TryFrom<Element> for OOB {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("x", ns::OUT_OF_BAND_DATA)?;

        Ok(Self {
            url: value
                .get_child("url", ns::OUT_OF_BAND_DATA)
                .ok_or(anyhow!(
                    "Missing element 'url' in Out-of-band data element."
                ))?
                .text(),
            desc: value
                .get_child("desc", ns::OUT_OF_BAND_DATA)
                .map(|e| e.text()),
        })
    }
}

impl From<OOB> for Element {
    fn from(value: OOB) -> Self {
        Element::builder("x", ns::OUT_OF_BAND_DATA)
            .append(Element::builder("url", ns::OUT_OF_BAND_DATA).append(value.url))
            .append_all(
                value
                    .desc
                    .map(|desc| Element::builder("desc", ns::OUT_OF_BAND_DATA).append(desc)),
            )
            .build()
    }
}

impl MessagePayload for OOB {}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Result;

    use super::*;

    #[test]
    fn test_deserialize_oob_without_desc() -> Result<()> {
        let xml = r#"<x xmlns='jabber:x:oob'>
          <url>http://www.jabber.org/images/psa-license.jpg</url>
        </x>
        "#;

        let elem = Element::from_str(xml)?;
        let oob = OOB::try_from(elem)?;

        assert_eq!(
            oob,
            OOB {
                url: "http://www.jabber.org/images/psa-license.jpg".to_string(),
                desc: None,
            }
        );

        Ok(())
    }

    #[test]
    fn test_deserialize_oob_with_desc() -> Result<()> {
        let xml = r#"<x xmlns='jabber:x:oob'>
          <url>http://www.jabber.org/images/psa-license.jpg</url>
          <desc>URL Description</desc>
        </x>
        "#;

        let elem = Element::from_str(xml)?;
        let oob = OOB::try_from(elem)?;

        assert_eq!(
            oob,
            OOB {
                url: "http://www.jabber.org/images/psa-license.jpg".to_string(),
                desc: Some("URL Description".to_string()),
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize_oob_without_desc() -> Result<()> {
        let oob = OOB {
            url: "http://www.jabber.org/images/psa-license.jpg".to_string(),
            desc: None,
        };

        let elem = Element::try_from(oob.clone())?;
        let parsed_oob = OOB::try_from(elem)?;

        assert_eq!(oob, parsed_oob);

        Ok(())
    }

    #[test]
    fn test_serialize_oob_with_desc() -> Result<()> {
        let oob = OOB {
            url: "http://www.jabber.org/images/psa-license.jpg".to_string(),
            desc: Some("URL Description".to_string()),
        };

        let elem = Element::try_from(oob.clone())?;
        let parsed_oob = OOB::try_from(elem)?;

        assert_eq!(oob, parsed_oob);

        Ok(())
    }
}
