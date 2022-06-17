use std::{collections::HashMap, convert::TryFrom};

use jid::BareJid;
use libstrophe::Stanza;
use std::str::FromStr;

pub struct Roster {
    pub groups: Vec<RosterGroup>,
}

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

pub enum RosterItemSubscription {
    None,
    To,
    From,
    Both,
}

pub struct RosterItem {
    pub jid: BareJid,
    pub subscription: RosterItemSubscription,
}

impl TryFrom<&str> for RosterItemSubscription {
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "to" => Ok(Self::To),
            "from" => Ok(Self::From),
            "both" => Ok(Self::Both),
            "none" => Ok(Self::None),
            _ => Err(()),
        }
    }

    type Error = ();
}

impl TryFrom<&Stanza> for Roster {
    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        let query = stanza.get_child_by_name("query").ok_or(())?;
        let mut groups: HashMap<String, RosterGroup> = HashMap::new();

        for item in query.children() {
            let group_name = item
                .get_child_by_name("group")
                .and_then(|g| g.text())
                .ok_or(())?;
            let jid = item
                .get_attribute("jid")
                .map(BareJid::from_str)
                .ok_or(())?
                .map_err(|_| ());
            let sub = item
                .get_attribute("subscription")
                .ok_or(())
                .and_then(|s| s.try_into());

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
