// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::{Availability, Presence};
use jid::{BareJid, Jid};
use std::collections::HashMap;

#[derive(Default)]
pub struct PresenceMap {
    map: HashMap<BareJid, Vec<PresenceEntry>>,
}

impl PresenceMap {
    #[allow(dead_code)]
    pub fn new() -> Self {
        PresenceMap::default()
    }

    pub fn update_presence(&mut self, from: &Jid, presence: Presence) {
        if presence.availability() == Availability::Unavailable {
            self.remove_presence(from)
        } else {
            self.insert_presence(from, presence)
        }
    }

    pub fn get_highest_presence(&self, jid: &BareJid) -> Option<&PresenceEntry> {
        self.map
            .get(jid)
            .and_then(|entries| entries.first())
            .map(|entry| entry)
    }
}

impl PresenceMap {
    fn remove_presence(&mut self, from: &Jid) {
        match from {
            Jid::Bare(jid) => {
                self.map.remove(jid);
            }
            Jid::Full(jid) => {
                if let Some(entries) = self.map.get_mut(&jid.to_bare()) {
                    entries.retain(|p| p.resource.as_deref() != Some(jid.resource_str()))
                }
            }
        }
    }

    fn insert_presence(&mut self, from: &Jid, presence: Presence) {
        let entries = self.map.entry(from.to_bare()).or_default();
        let resource = from.resource_str();
        entries.retain(|entry| entry.resource.as_deref() != resource && entry.resource.is_some());
        let idx = entries
            .iter()
            .position(|entry| entry.presence.priority <= presence.priority)
            .unwrap_or(entries.len());
        entries.insert(
            idx,
            PresenceEntry {
                resource: resource.map(ToString::to_string),
                presence,
            },
        );
    }
}

#[derive(PartialEq, Debug)]
pub struct PresenceEntry {
    pub resource: Option<String>,
    pub presence: Presence,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::presence;
    use prose_xmpp::jid_str;

    #[test]
    fn test_update_with_eq_priority() {
        let user = jid_str!("a@prose.org").into_bare();

        let mut map = PresenceMap::new();

        map.update_presence(&jid_str!("a@prose.org/r1"), p(1));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );

        map.update_presence(&jid_str!("a@prose.org/r2"), p(1));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r2".to_string())
        );
    }

    #[test]
    fn test_update_with_lower_priority() {
        let user = jid_str!("a@prose.org").into_bare();

        let mut map = PresenceMap::new();

        map.update_presence(&jid_str!("a@prose.org/r1"), p(2));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );

        map.update_presence(&jid_str!("a@prose.org/r2"), p(1));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );
    }

    #[test]
    fn test_update_with_higher_priority() {
        let user = jid_str!("a@prose.org").into_bare();

        let mut map = PresenceMap::new();

        map.update_presence(&jid_str!("a@prose.org/r1"), p(1));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );

        map.update_presence(&jid_str!("a@prose.org/r2"), p(2));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r2".to_string())
        );
    }

    #[test]
    fn test_update_with_unavailable() {
        let user = jid_str!("a@prose.org").into_bare();

        let mut map = PresenceMap::new();

        map.update_presence(&jid_str!("a@prose.org/r1"), p(1));
        map.update_presence(&jid_str!("a@prose.org/r2"), p(2));

        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r2".to_string())
        );

        map.update_presence(&jid_str!("a@prose.org/r2"), unavailable());
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );

        map.update_presence(&jid_str!("a@prose.org/r1"), unavailable());
        assert_eq!(map.get_highest_presence(&user), None);
    }

    #[test]
    fn test_update_with_bare_jid() {
        let user = jid_str!("a@prose.org").into_bare();

        let mut map = PresenceMap::new();

        map.update_presence(&jid_str!("a@prose.org"), p(1));
        assert_eq!(map.get_highest_presence(&user).unwrap().resource, None);
        assert_eq!(
            map.get_highest_presence(&user).unwrap().presence.priority,
            1
        );

        map.update_presence(&jid_str!("a@prose.org"), p(2));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().presence.priority,
            2
        );
    }

    #[test]
    fn test_full_jid_replaces_bare_jid() {
        let user = jid_str!("a@prose.org").into_bare();

        let mut map = PresenceMap::new();

        map.update_presence(&jid_str!("a@prose.org"), p(1));
        map.update_presence(&jid_str!("a@prose.org/r1"), p(2));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );

        map.update_presence(&jid_str!("a@prose.org/r1"), unavailable());
        assert_eq!(map.get_highest_presence(&user), None);
    }

    #[test]
    fn test_update_with_unavailable_bare_jid() {
        let user = jid_str!("a@prose.org").into_bare();

        let mut map = PresenceMap::new();

        map.update_presence(&jid_str!("a@prose.org/r1"), p(1));
        map.update_presence(&jid_str!("a@prose.org/r2"), p(2));

        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r2".to_string())
        );

        map.update_presence(&jid_str!("a@prose.org"), p(2));
        assert_eq!(map.get_highest_presence(&user).unwrap().resource, None);
    }

    #[test]
    fn test_multiple_users() {
        let user1 = jid_str!("a@prose.org").into_bare();
        let user2 = jid_str!("b@prose.org").into_bare();

        let mut map = PresenceMap::new();

        map.update_presence(&jid_str!("a@prose.org/ra1"), p(1));
        map.update_presence(&jid_str!("b@prose.org/ra2"), p(1));

        assert_eq!(
            map.get_highest_presence(&user1).unwrap().resource,
            Some("ra1".to_string())
        );
        assert_eq!(
            map.get_highest_presence(&user2).unwrap().resource,
            Some("ra2".to_string())
        );
    }

    fn p(priority: i8) -> Presence {
        Presence {
            kind: None,
            show: None,
            status: None,
            priority,
        }
    }

    fn unavailable() -> Presence {
        Presence {
            kind: Some(presence::Type(xmpp_parsers::presence::Type::Unavailable)),
            show: None,
            status: None,
            priority: 0,
        }
    }
}
