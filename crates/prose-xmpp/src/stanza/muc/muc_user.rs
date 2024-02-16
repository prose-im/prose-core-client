// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{ns, ElementExt, RequestError};
use jid::BareJid;
use minidom::Element;
use std::str::FromStr;
use xmpp_parsers::message::MessagePayload;
use xmpp_parsers::muc::user::{Item, Status};
use xmpp_parsers::presence::PresencePayload;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct MucUser {
    /// List of statuses applying to this item.
    pub status: Vec<Status>,

    /// List of items.
    pub items: Vec<Item>,

    /// Has the room been destroyed?
    pub destroy: Option<Destroy>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Destroy {
    pub jid: Option<BareJid>,
    pub reason: Option<String>,
}

impl MucUser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status(mut self, status: impl IntoIterator<Item = Status>) -> Self {
        self.status = status.into_iter().collect();
        self
    }

    pub fn with_item(mut self, item: Item) -> Self {
        self.items.push(item);
        self
    }

    pub fn with_items(mut self, items: impl IntoIterator<Item = Item>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn with_destroy(mut self, destroy: Destroy) -> Self {
        self.destroy = Some(destroy);
        self
    }
}

impl MessagePayload for MucUser {}
impl PresencePayload for MucUser {}

impl TryFrom<Element> for MucUser {
    type Error = RequestError;

    fn try_from(root: Element) -> Result<Self, Self::Error> {
        root.expect_is("x", ns::MUC_USER)?;

        let mut user = MucUser::default();

        for child in root.children() {
            match child {
                _ if child.is("item", ns::MUC_USER) => {
                    user.items.push(Item::try_from(child.clone())?);
                }
                _ if child.is("status", ns::MUC_USER) => {
                    user.status.push(Status::try_from(child.clone())?);
                }
                _ if child.is("destroy", ns::MUC_USER) => {
                    user.destroy = Some(Destroy::try_from(child.clone())?);
                }
                _ => {
                    return Err(RequestError::Generic {
                        msg: format!(
                            "Encountered unexpected payload {} in muc query.",
                            child.name()
                        ),
                    })
                }
            }
        }

        Ok(user)
    }
}

impl From<MucUser> for Element {
    fn from(value: MucUser) -> Self {
        Element::builder("x", ns::MUC_USER)
            .append_all(value.status)
            .append_all(value.items)
            .append_all(value.destroy)
            .build()
    }
}

impl TryFrom<Element> for Destroy {
    type Error = RequestError;

    fn try_from(root: Element) -> Result<Self, Self::Error> {
        root.expect_is("destroy", ns::MUC_USER)?;

        Ok(Destroy {
            jid: root.attr("jid").map(BareJid::from_str).transpose()?,
            reason: root
                .get_child("reason", ns::MUC_USER)
                .map(|node| node.text()),
        })
    }
}

impl From<Destroy> for Element {
    fn from(value: Destroy) -> Self {
        Element::builder("destroy", ns::MUC_USER)
            .attr("jid", value.jid)
            .append_all(value.reason.map(|reason| {
                Element::builder("reason", ns::MUC_USER)
                    .append(reason)
                    .build()
            }))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bare;
    use anyhow::Result;
    use std::str::FromStr;
    use xmpp_parsers::muc::user::{Affiliation, Role};

    #[test]
    fn test_deserialize_muc_user() -> Result<()> {
        let xml = r#"<x xmlns='http://jabber.org/protocol/muc#user'>
            <status code='101'/>
            <status code='102'/>
            <item affiliation='member' role='moderator'/>
            <destroy jid='coven@chat.shakespeare.lit'>
                <reason>Macbeth doth come.</reason>
            </destroy>
        </x>
        "#;

        let elem = Element::from_str(xml)?;
        let user = MucUser::try_from(elem)?;

        assert_eq!(
            user,
            MucUser {
                status: vec![
                    Status::AffiliationChange,
                    Status::ConfigShowsUnavailableMembers
                ],
                items: vec![Item {
                    affiliation: Affiliation::Member,
                    jid: None,
                    nick: None,
                    role: Role::Moderator,
                    actor: None,
                    continue_: None,
                    reason: None,
                }],
                destroy: Some(Destroy {
                    jid: Some(bare!("coven@chat.shakespeare.lit")),
                    reason: Some("Macbeth doth come.".to_string())
                }),
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize_muc_user() -> Result<()> {
        let user = MucUser {
            status: vec![
                Status::AffiliationChange,
                Status::ConfigShowsUnavailableMembers,
            ],
            items: vec![Item {
                affiliation: Affiliation::Member,
                jid: None,
                nick: None,
                role: Role::Moderator,
                actor: None,
                continue_: None,
                reason: None,
            }],
            destroy: Some(Destroy {
                jid: Some(bare!("coven@chat.shakespeare.lit")),
                reason: Some("Macbeth doth come.".to_string()),
            }),
        };

        let elem = Element::try_from(user.clone())?;
        let parsed_user = MucUser::try_from(elem)?;

        assert_eq!(user, parsed_user);

        Ok(())
    }
}
