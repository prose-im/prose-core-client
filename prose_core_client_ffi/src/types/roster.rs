// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use libstrophe::Stanza;
use std::str::FromStr;
use std::{collections::HashMap, convert::TryFrom};
use strum_macros::{Display, EnumString};

static DEFAULT_GROUP_NAME: &str = "_default_group_";

#[derive(Debug, PartialEq)]
pub struct Roster {
    pub groups: Vec<RosterGroup>,
}

#[derive(Debug, PartialEq)]
pub struct RosterGroup {
    pub name: String,
    pub items: Vec<RosterItem>,
}

impl RosterGroup {
    fn new(name: String) -> Self {
        RosterGroup {
            name: name,
            items: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum RosterItemSubscription {
    None,
    To,
    From,
    Both,
}

#[derive(Debug, PartialEq)]
pub struct RosterItem {
    pub jid: BareJid,
    pub subscription: RosterItemSubscription,
}

impl TryFrom<&Stanza> for Roster {
    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        let query = stanza.get_child_by_name("query").ok_or(())?;
        let mut groups: HashMap<String, RosterGroup> = HashMap::new();

        for item in query.children() {
            let group_name = item
                .get_child_by_name("group")
                .and_then(|g| g.text())
                .unwrap_or_else(|| DEFAULT_GROUP_NAME.to_string());
            let jid = item
                .get_attribute("jid")
                .map(BareJid::from_str)
                .ok_or(())?
                .map_err(|_| ());
            let sub = item
                .get_attribute("subscription")
                .ok_or(())
                .and_then(|s| s.parse::<RosterItemSubscription>().map_err(|_| ()));
            let item = RosterItem {
                jid: jid?,
                subscription: sub?,
            };
            groups
                .entry(group_name.clone())
                .or_insert_with(|| RosterGroup::new(group_name.clone()))
                .items
                .push(item);
        }

        Ok(Roster {
            groups: groups.into_values().collect(),
        })
    }

    type Error = ();
}

#[cfg(test)]
mod tests {
    use libstrophe::Stanza;

    use super::*;

    #[test]
    fn test_roster_with_groups() {
        let roster = r#"
        <iq id="roster1" type="result" to="marc@prose.org/5vvvDQpY">
          <query xmlns="jabber:iq:roster" ver="7"><item jid="valerian@prose.org" subscription="both"><group>Contacts</group></item><item ask="subscribe" jid="remi@prose.org" subscription="none"><group>Contacts</group></item></query>
        </iq>
        "#;

        let stanza = Stanza::from_str(roster);
        let roster = Roster::try_from(&stanza).unwrap();

        assert_eq!(
            roster,
            Roster {
                groups: vec![RosterGroup {
                    name: "Contacts".to_string(),
                    items: vec![
                        RosterItem {
                            jid: BareJid::from_str("valerian@prose.org").unwrap(),
                            subscription: RosterItemSubscription::Both
                        },
                        RosterItem {
                            jid: BareJid::from_str("remi@prose.org").unwrap(),
                            subscription: RosterItemSubscription::None
                        }
                    ]
                }]
            }
        );
    }

    #[test]
    fn test_roster_without_groups() {
        let roster = r#"
        <iq id="roster1" type="result" to="valerian@prose.org/c4HSNMnR">
          <query xmlns="jabber:iq:roster" ver="12"><item jid="valerian@valeriansaliou.name" subscription="both"/><item jid="marc@prose.org" subscription="both"/></query>
        </iq>
        "#;

        let stanza = Stanza::from_str(roster);
        let roster = Roster::try_from(&stanza).unwrap();

        assert_eq!(
            roster,
            Roster {
                groups: vec![RosterGroup {
                    name: "_default_group_".to_string(),
                    items: vec![
                        RosterItem {
                            jid: BareJid::from_str("valerian@valeriansaliou.name").unwrap(),
                            subscription: RosterItemSubscription::Both
                        },
                        RosterItem {
                            jid: BareJid::from_str("marc@prose.org").unwrap(),
                            subscription: RosterItemSubscription::Both
                        }
                    ]
                }]
            }
        );
    }
}
