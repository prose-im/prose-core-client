// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use crate::domain::shared::models::{Availability, UserId, UserOrResourceId, UserResourceId};
use crate::domain::user_info::models::Presence;

#[derive(Default)]
pub struct PresenceMap {
    map: HashMap<UserId, Vec<PresenceEntry>>,
}

impl PresenceMap {
    #[allow(dead_code)]
    pub fn new() -> Self {
        PresenceMap::default()
    }

    pub fn update_presence(&mut self, from: &UserOrResourceId, presence: Presence) {
        if presence.availability == Availability::Unavailable {
            self.remove_presence(from)
        } else {
            self.insert_presence(from, presence)
        }
    }

    pub fn get_highest_presence(&self, jid: &UserId) -> Option<&PresenceEntry> {
        self.map
            .get(jid)
            .and_then(|entries| entries.first())
            .map(|entry| entry)
    }

    pub fn clear(&mut self) {
        self.map.clear()
    }
}

impl PresenceMap {
    fn remove_presence(&mut self, id: &UserOrResourceId) {
        match id {
            UserOrResourceId::User(id) => {
                self.map.remove(id);
            }
            UserOrResourceId::UserResource(id) => {
                if let Some(entries) = self.map.get_mut(&id.to_user_id()) {
                    entries.retain(|p| p.resource.as_deref() != Some(id.resource()))
                }
            }
        }
    }

    fn insert_presence(&mut self, id: &UserOrResourceId, presence: Presence) {
        let entries = self.map.entry(id.to_user_id()).or_default();
        let resource = id.resource_str();
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
    use crate::{user_id, user_resource_id};

    use super::*;

    #[test]
    fn test_update_with_eq_priority() {
        let user = user_id!("a@prose.org");

        let mut map = PresenceMap::new();

        map.update_presence(&user_resource_id!("a@prose.org/r1").into(), p(1));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );

        map.update_presence(&user_resource_id!("a@prose.org/r2").into(), p(1));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r2".to_string())
        );
    }

    #[test]
    fn test_update_with_lower_priority() {
        let user = user_id!("a@prose.org");

        let mut map = PresenceMap::new();

        map.update_presence(&user_resource_id!("a@prose.org/r1").into(), p(2));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );

        map.update_presence(&user_resource_id!("a@prose.org/r2").into(), p(1));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );
    }

    #[test]
    fn test_update_with_higher_priority() {
        let user = user_id!("a@prose.org");

        let mut map = PresenceMap::new();

        map.update_presence(&user_resource_id!("a@prose.org/r1").into(), p(1));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );

        map.update_presence(&user_resource_id!("a@prose.org/r2").into(), p(2));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r2".to_string())
        );
    }

    #[test]
    fn test_update_with_unavailable() {
        let user = user_id!("a@prose.org");

        let mut map = PresenceMap::new();

        map.update_presence(&user_resource_id!("a@prose.org/r1").into(), p(1));
        map.update_presence(&user_resource_id!("a@prose.org/r2").into(), p(2));

        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r2".to_string())
        );

        map.update_presence(
            &user_resource_id!("a@prose.org/r2").into(),
            Presence::default(),
        );
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );

        map.update_presence(
            &user_resource_id!("a@prose.org/r1").into(),
            Presence::default(),
        );
        assert_eq!(map.get_highest_presence(&user), None);
    }

    #[test]
    fn test_update_with_bare_jid() {
        let user = user_id!("a@prose.org");

        let mut map = PresenceMap::new();

        map.update_presence(&user_resource_id!("a@prose.org").into(), p(1));
        assert_eq!(map.get_highest_presence(&user).unwrap().resource, None);
        assert_eq!(
            map.get_highest_presence(&user).unwrap().presence.priority,
            1
        );

        map.update_presence(&user_resource_id!("a@prose.org").into(), p(2));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().presence.priority,
            2
        );
    }

    #[test]
    fn test_full_jid_replaces_bare_jid() {
        let user = user_id!("a@prose.org");

        let mut map = PresenceMap::new();

        map.update_presence(&user_resource_id!("a@prose.org").into(), p(1));
        map.update_presence(&user_resource_id!("a@prose.org/r1").into(), p(2));
        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r1".to_string())
        );

        map.update_presence(
            &user_resource_id!("a@prose.org/r1").into(),
            Presence::default(),
        );
        assert_eq!(map.get_highest_presence(&user), None);
    }

    #[test]
    fn test_update_with_unavailable_bare_jid() {
        let user = user_id!("a@prose.org");

        let mut map = PresenceMap::new();

        map.update_presence(&user_resource_id!("a@prose.org/r1").into(), p(1));
        map.update_presence(&user_resource_id!("a@prose.org/r2").into(), p(2));

        assert_eq!(
            map.get_highest_presence(&user).unwrap().resource,
            Some("r2".to_string())
        );

        map.update_presence(&user_resource_id!("a@prose.org").into(), p(2));
        assert_eq!(map.get_highest_presence(&user).unwrap().resource, None);
    }

    #[test]
    fn test_multiple_users() {
        let user1 = user_id!("a@prose.org");
        let user2 = user_id!("b@prose.org");

        let mut map = PresenceMap::new();

        map.update_presence(&user_resource_id!("a@prose.org/ra1").into(), p(1));
        map.update_presence(&user_resource_id!("b@prose.org/ra2").into(), p(1));

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
            priority,
            availability: Availability::Available,
            status: None,
        }
    }
}
