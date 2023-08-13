// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::ns;
use crate::util::ElementExt;
use anyhow::Context;
use minidom::Element;
use xmpp_parsers::iq::{IqGetPayload, IqResultPayload};

pub struct LastActivityRequest;

#[derive(Debug, PartialEq)]
pub struct LastActivityResponse {
    pub seconds: u64,
    pub status: Option<String>,
}

impl IqGetPayload for LastActivityRequest {}

impl IqResultPayload for LastActivityResponse {}

impl TryFrom<Element> for LastActivityRequest {
    type Error = anyhow::Error;

    fn try_from(root: Element) -> Result<Self, Self::Error> {
        root.expect_is("query", ns::LAST_ACTIVITY)?;
        Ok(LastActivityRequest {})
    }
}

impl From<LastActivityRequest> for Element {
    fn from(_value: LastActivityRequest) -> Self {
        Element::builder("query", ns::LAST_ACTIVITY).build()
    }
}

impl TryFrom<Element> for LastActivityResponse {
    type Error = anyhow::Error;

    fn try_from(root: Element) -> Result<Self, Self::Error> {
        Ok(LastActivityResponse {
            seconds: root
                .req_attr("seconds")?
                .parse::<u64>()
                .context("Failed to parse seconds in LastActivityResponse")?,
            status: root.texts().next().map(|s| s.to_string()),
        })
    }
}

impl From<LastActivityResponse> for Element {
    fn from(value: LastActivityResponse) -> Self {
        Element::builder("query", ns::LAST_ACTIVITY)
            .attr("seconds", value.seconds)
            .append_all(value.status)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::str::FromStr;

    #[test]
    fn test_deserialize_last_activity() -> Result<()> {
        assert_eq!(
            LastActivityResponse::try_from(Element::from_str(
                "<query xmlns='jabber:iq:last' seconds='903'>Heading Home</query>"
            )?)?,
            LastActivityResponse {
                seconds: 903,
                status: Some("Heading Home".to_string())
            }
        );

        assert_eq!(
            LastActivityResponse::try_from(Element::from_str(
                "<query xmlns='jabber:iq:last' seconds='123'/>"
            )?)?,
            LastActivityResponse {
                seconds: 123,
                status: None
            }
        );

        Ok(())
    }
}
