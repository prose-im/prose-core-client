// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use anyhow::Result;
use jid::BareJid;
use minidom::Element;
use xmpp_parsers::message::MessagePayload;
use xmpp_parsers::muc::user::Affiliation;

use crate::{ns, ElementExt, ParseError};

#[derive(Debug, Clone, PartialEq)]
pub struct MucInvite {
    pub jid: BareJid,
    pub affiliation: Affiliation,
    pub reason: Option<String>,
}

impl TryFrom<Element> for MucInvite {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self> {
        value.expect_is("x", ns::MUC_USER)?;

        let Some(item) = value.get_child("item", ns::MUC_USER) else {
            return Err(ParseError::Generic {
                msg: "Missing item in MucUser".to_string(),
            }
            .into());
        };

        let jid = BareJid::from_str(item.attr_req("jid")?)?;
        let affiliation = Affiliation::from_str(item.attr_req("affiliation")?)?;
        let reason = item.children().find_map(|child| {
            if !child.is("reason", ns::MUC_USER) {
                return None;
            }
            let text = child.text();
            (!text.is_empty()).then_some(text)
        });

        Ok(Self {
            jid,
            affiliation,
            reason,
        })
    }
}

impl From<MucInvite> for Element {
    fn from(value: MucInvite) -> Self {
        let affiliation = match &value.affiliation {
            Affiliation::Owner => "owner",
            Affiliation::Admin => "admin",
            Affiliation::Member => "member",
            Affiliation::Outcast => "outcast",
            Affiliation::None => "none",
        };

        Element::builder("x", ns::MUC_USER)
            .append(
                Element::builder("item", ns::MUC_USER)
                    .attr("jid", value.jid)
                    .attr("affiliation", affiliation)
                    .append_all(value.reason.map(|reason| {
                        Element::builder("reason", ns::MUC_USER)
                            .append(reason)
                            .build()
                    })),
            )
            .build()
    }
}

impl MessagePayload for MucInvite {}

#[cfg(test)]
mod tests {
    use crate::bare;

    use super::*;

    #[test]
    fn test_deserialize() -> Result<()> {
        let xml = r#"<x xmlns='http://jabber.org/protocol/muc#user'>
        <item jid='hello@prose.org' affiliation='member' role='participant'>
            <reason>Invited by world@prose.org/res</reason>
        </item>
      </x>"#;

        let elem = Element::from_str(xml)?;
        let user = MucInvite::try_from(elem)?;

        assert_eq!(
            user,
            MucInvite {
                jid: bare!("hello@prose.org"),
                affiliation: Affiliation::Member,
                reason: Some("Invited by world@prose.org/res".to_string()),
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize() -> Result<()> {
        let user = MucInvite {
            jid: bare!("user@prose.org"),
            affiliation: Affiliation::Owner,
            reason: Some("User was invited by other_user@prose.org".to_string()),
        };

        let elem = Element::from(user.clone());
        let parsed_user = MucInvite::try_from(elem)?;

        assert_eq!(parsed_user, user);

        Ok(())
    }
}
