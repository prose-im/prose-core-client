// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use anyhow::Result;
use jid::Jid;
use minidom::Element;
use xmpp_parsers::message::MessagePayload;
use xmpp_parsers::muc::user::{Affiliation, Role};

use crate::{ns, ElementExt, ParseError};

#[derive(Debug, Clone, PartialEq)]
pub struct MucUser {
    pub jid: Option<Jid>,
    pub affiliation: Affiliation,
    pub role: Role,
}

impl TryFrom<Element> for MucUser {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self> {
        value.expect_is("x", ns::MUC_USER)?;

        let Some(item) = value.get_child("item", ns::MUC_USER) else {
            return Err(ParseError::Generic {
                msg: "Missing item in MucUser".to_string(),
            }
            .into());
        };

        let jid = item
            .attr("jid")
            .map(|jid_str| Jid::from_str(jid_str))
            .transpose()?;
        let affiliation = Affiliation::from_str(item.attr_req("affiliation")?)?;
        let role = Role::from_str(item.attr_req("role")?)?;

        Ok(Self {
            jid,
            affiliation,
            role,
        })
    }
}

impl From<MucUser> for Element {
    fn from(value: MucUser) -> Self {
        let affiliation = match &value.affiliation {
            Affiliation::Owner => "owner",
            Affiliation::Admin => "admin",
            Affiliation::Member => "member",
            Affiliation::Outcast => "outcast",
            Affiliation::None => "none",
        };

        let role = match &value.role {
            Role::Moderator => "moderator",
            Role::Participant => "participant",
            Role::Visitor => "visitor",
            Role::None => "none",
        };

        Element::builder("x", ns::MUC_USER)
            .append(
                Element::builder("item", ns::MUC_USER)
                    .attr("jid", value.jid)
                    .attr("affiliation", affiliation)
                    .attr("role", role),
            )
            .build()
    }
}

impl MessagePayload for MucUser {}

#[cfg(test)]
mod tests {
    use crate::jid;

    use super::*;

    #[test]
    fn test_deserialize_with_jid() -> Result<()> {
        let xml = r#"<x xmlns='http://jabber.org/protocol/muc#user'>
        <item jid='hello@prose.org' affiliation='none' role='participant' />
      </x>"#;

        let elem = Element::from_str(xml)?;
        let user = MucUser::try_from(elem)?;

        assert_eq!(
            user,
            MucUser {
                jid: Some(jid!("hello@prose.org")),
                affiliation: Affiliation::None,
                role: Role::Participant,
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize_with_jid() -> Result<()> {
        let user = MucUser {
            jid: Some(jid!("hello@prose.org")),
            affiliation: Affiliation::None,
            role: Role::Moderator,
        };

        let elem = Element::from(user.clone());
        let parsed_user = MucUser::try_from(elem)?;

        assert_eq!(user, parsed_user);

        Ok(())
    }

    #[test]
    fn test_deserialize_without_jid() -> Result<()> {
        let xml = r#"<x xmlns='http://jabber.org/protocol/muc#user'>
        <item affiliation='none' role='participant' />
      </x>"#;

        let elem = Element::from_str(xml)?;
        let user = MucUser::try_from(elem)?;

        assert_eq!(
            user,
            MucUser {
                jid: None,
                affiliation: Affiliation::None,
                role: Role::Participant,
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize_without_jid() -> Result<()> {
        let user = MucUser {
            jid: None,
            affiliation: Affiliation::Owner,
            role: Role::None,
        };

        let elem = Element::from(user.clone());
        let parsed_user = MucUser::try_from(elem)?;

        assert_eq!(user, parsed_user);

        Ok(())
    }
}
