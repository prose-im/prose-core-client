// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::ns;
use crate::util::{ElementBuilderExt, ElementExt};
use jid::BareJid;
use minidom::Element;
use std::str::FromStr;
use xmpp_parsers::message::MessagePayload;

#[derive(Debug, PartialEq, Clone)]
pub struct DirectInvite {
    pub jid: BareJid,
    pub password: Option<String>,
    pub reason: Option<String>,
    pub r#continue: Option<bool>,
    pub thread: Option<String>,
}

impl TryFrom<Element> for DirectInvite {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("x", ns::DIRECT_MUC_INVITATIONS)?;

        Ok(DirectInvite {
            jid: BareJid::from_str(value.attr_req("jid")?)?,
            password: value.attr("password").map(ToString::to_string),
            reason: value.attr("reason").map(ToString::to_string),
            r#continue: value.attr_bool("continue")?,
            thread: value.attr("thread").map(ToString::to_string),
        })
    }
}

impl From<DirectInvite> for Element {
    fn from(value: DirectInvite) -> Self {
        Element::builder("x", ns::DIRECT_MUC_INVITATIONS)
            .attr("jid", value.jid)
            .attr("password", value.password)
            .attr("reason", value.reason)
            .attr_bool_opt("continue", value.r#continue)
            .attr("thread", value.thread)
            .build()
    }
}

impl MessagePayload for DirectInvite {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jid;
    use anyhow::Result;

    #[test]
    fn test_deserialize_full_direct_invite() -> Result<()> {
        let xml = r#"<x xmlns='jabber:x:conference'
          continue='true'
          jid='darkcave@macbeth.shakespeare.lit'
          password='cauldronburn'
          reason='Hey Hecate, this is the place for all good witches!'
          thread='e0ffe42b28561960c6b12b944a092794b9683a38'/>
        "#;

        let elem = Element::from_str(xml)?;
        let invite = DirectInvite::try_from(elem)?;

        assert_eq!(
            invite,
            DirectInvite {
                jid: jid!("darkcave@macbeth.shakespeare.lit").into_bare(),
                password: Some("cauldronburn".to_string()),
                reason: Some("Hey Hecate, this is the place for all good witches!".to_string()),
                r#continue: Some(true),
                thread: Some("e0ffe42b28561960c6b12b944a092794b9683a38".to_string()),
            }
        );

        Ok(())
    }

    #[test]
    fn test_deserialize_minimal_direct_invite() -> Result<()> {
        let xml = "<x xmlns='jabber:x:conference' jid='darkcave@macbeth.shakespeare.lit'/>";

        let elem = Element::from_str(xml)?;
        let invite = DirectInvite::try_from(elem)?;

        assert_eq!(
            invite,
            DirectInvite {
                jid: jid!("darkcave@macbeth.shakespeare.lit").into_bare(),
                password: None,
                reason: None,
                r#continue: None,
                thread: None,
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize_full_direct_invite() -> Result<()> {
        let invite = DirectInvite {
            jid: jid!("darkcave@macbeth.shakespeare.lit").into_bare(),
            password: Some("cauldronburn".to_string()),
            reason: Some("Hey Hecate, this is the place for all good witches!".to_string()),
            r#continue: Some(true),
            thread: Some("e0ffe42b28561960c6b12b944a092794b9683a38".to_string()),
        };

        let elem = Element::try_from(invite.clone())?;
        let parsed_invite = DirectInvite::try_from(elem)?;

        assert_eq!(invite, parsed_invite);

        Ok(())
    }

    #[test]
    fn test_serialize_minimal_direct_invite() -> Result<()> {
        let invite = DirectInvite {
            jid: jid!("darkcave@macbeth.shakespeare.lit").into_bare(),
            password: None,
            reason: None,
            r#continue: None,
            thread: None,
        };

        let elem = Element::try_from(invite.clone())?;
        let parsed_invite = DirectInvite::try_from(elem)?;

        assert_eq!(invite, parsed_invite);

        Ok(())
    }
}
