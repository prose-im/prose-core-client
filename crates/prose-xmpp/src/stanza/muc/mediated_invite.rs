// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::ns;
use crate::util::ElementExt;
use jid::Jid;
use minidom::Element;
use std::str::FromStr;
use xmpp_parsers::message::MessagePayload;

#[derive(Debug, PartialEq, Clone)]
pub struct MediatedInvite {
    pub invites: Vec<Invite>,
    pub password: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Invite {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub reason: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Continue {
    pub thread: Option<String>,
}

impl From<MediatedInvite> for Element {
    fn from(value: MediatedInvite) -> Self {
        Element::builder("x", ns::MUC_USER)
            .append_all(value.invites)
            .append_all(
                value
                    .password
                    .map(|password| Element::builder("password", ns::MUC_USER).append(password)),
            )
            .build()
    }
}

impl TryFrom<Element> for MediatedInvite {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("x", ns::MUC_USER)?;

        let mut password = None;
        let mut invites = vec![];

        for child in value.children() {
            match child {
                _ if child.is("invite", ns::MUC_USER) => {
                    invites.push(Invite::try_from(child.clone())?)
                }
                _ if child.is("password", ns::MUC_USER) => password = Some(child.text()),
                _ => (),
            }
        }

        Ok(MediatedInvite { invites, password })
    }
}

impl MessagePayload for MediatedInvite {}

impl From<Invite> for Element {
    fn from(value: Invite) -> Self {
        Element::builder("invite", ns::MUC_USER)
            .attr("from", value.from)
            .attr("to", value.to)
            .append_all(
                value
                    .reason
                    .map(|reason| Element::builder("reason", ns::MUC_USER).append(reason)),
            )
            .build()
    }
}

impl TryFrom<Element> for Invite {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("invite", ns::MUC_USER)?;

        Ok(Invite {
            from: value.attr("from").map(FromStr::from_str).transpose()?,
            to: value.attr("to").map(FromStr::from_str).transpose()?,
            reason: value
                .get_child("reason", ns::MUC_USER)
                .map(|child| child.text()),
        })
    }
}

impl From<Continue> for Element {
    fn from(value: Continue) -> Self {
        Element::builder("continue", ns::MUC_USER)
            .attr("thread", value.thread)
            .build()
    }
}

impl TryFrom<Element> for Continue {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("continue", ns::MUC_USER)?;

        Ok(Continue {
            thread: value.attr("thread").map(ToString::to_string),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::jid;
    use anyhow::Result;

    #[test]
    fn test_deserialize_mediated_invite() -> Result<()> {
        let xml = r#"<x xmlns='http://jabber.org/protocol/muc#user'>
        <invite from='crone1@shakespeare.lit/desktop'>
          <reason>Hey Hecate, this is the place for all good witches!</reason>
        </invite>
        <password>cauldronburn</password>
        </x>
        "#;

        let elem = Element::from_str(xml)?;
        let invite = MediatedInvite::try_from(elem)?;

        assert_eq!(
            invite,
            MediatedInvite {
                invites: vec![Invite {
                    from: Some(jid!("crone1@shakespeare.lit/desktop")),
                    to: None,
                    reason: Some("Hey Hecate, this is the place for all good witches!".to_string()),
                }],
                password: Some("cauldronburn".to_string())
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize_mediated_invite() -> Result<()> {
        let invite = MediatedInvite {
            invites: vec![Invite {
                from: Some(jid!("crone1@shakespeare.lit/desktop")),
                to: None,
                reason: Some("Hey Hecate, this is the place for all good witches!".to_string()),
            }],
            password: Some("cauldronburn".to_string()),
        };

        let elem = Element::try_from(invite.clone())?;
        let parsed_invite = MediatedInvite::try_from(elem)?;

        assert_eq!(invite, parsed_invite);

        Ok(())
    }
}
