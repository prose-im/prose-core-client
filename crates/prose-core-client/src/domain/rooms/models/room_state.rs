// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use jid::{BareJid, Jid};

use crate::domain::rooms::models::{ComposeState, RoomAffiliation};
use crate::domain::shared::models::UserBasicInfo;
use crate::util::jid_ext::BareJidExt;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct RoomState {
    /// The name of the room.
    pub name: Option<String>,
    /// The room's subject.
    pub topic: Option<String>,
    /// The occupants of the room. The key is either the user's FullJid in a MUC room or the user's
    /// BareJid in direct message room.
    pub occupants: HashMap<Jid, Occupant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Occupant {
    /// The real JID of the occupant. Only available in non-anonymous rooms.
    pub jid: Option<BareJid>,
    pub name: Option<String>,
    pub affiliation: RoomAffiliation,
    pub compose_state: ComposeState,
    pub compose_state_updated: DateTime<Utc>,
}

impl Default for Occupant {
    fn default() -> Self {
        Self {
            jid: None,
            name: None,
            affiliation: Default::default(),
            compose_state: ComposeState::Idle,
            compose_state_updated: Default::default(),
        }
    }
}

impl RoomState {
    pub fn insert_occupant(
        &mut self,
        jid: &Jid,
        real_jid: Option<&BareJid>,
        name: Option<&str>,
        affiliation: &RoomAffiliation,
    ) {
        let occupant = self.occupants.entry(jid.clone()).or_default();
        occupant.jid = real_jid.cloned();
        occupant.name = name.map(ToString::to_string);
        occupant.affiliation = affiliation.clone();
    }

    pub fn set_occupant_compose_state(
        &mut self,
        occupant_jid: &Jid,
        timestamp: &DateTime<Utc>,
        compose_state: ComposeState,
    ) {
        self.occupants
            .entry(occupant_jid.clone())
            .and_modify(|occupant| {
                occupant.compose_state = compose_state;
                occupant.compose_state_updated = timestamp.clone()
            });
    }

    /// Returns the real JIDs of all composing users that started composing after `started_after`.
    /// If we don't have a real JID for a composing user they are excluded from the list.
    pub fn composing_users(&self, started_after: DateTime<Utc>) -> Vec<UserBasicInfo> {
        let mut composing_occupants = self
            .occupants
            .values()
            .filter_map(|occupant| {
                if occupant.compose_state != ComposeState::Composing
                    || occupant.compose_state_updated <= started_after
                    || occupant.jid.is_none()
                {
                    return None;
                }
                Some(occupant.clone())
            })
            .collect::<Vec<_>>();

        composing_occupants.sort_by_key(|o| o.compose_state_updated);

        composing_occupants
            .into_iter()
            .filter_map(|occupant| {
                let Some(jid) = &occupant.jid else {
                    return None;
                };

                Some(UserBasicInfo {
                    name: occupant
                        .name
                        .clone()
                        .unwrap_or_else(|| jid.to_display_name()),
                    jid: jid.clone(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use prose_xmpp::{bare, jid};

    use super::*;

    #[test]
    fn test_insert_occupant() {
        let mut state = RoomState::default();
        assert!(state.occupants.is_empty());

        state.insert_occupant(
            &jid!("room@prose.org/a"),
            Some(&bare!("a@prose.org")),
            None,
            &RoomAffiliation::Owner,
        );
        state.insert_occupant(&jid!("b@prose.org"), None, None, &RoomAffiliation::Member);

        assert_eq!(state.occupants.len(), 2);
        assert_eq!(
            state.occupants.get(&jid!("room@prose.org/a")).unwrap(),
            &Occupant {
                jid: Some(bare!("a@prose.org")),
                affiliation: RoomAffiliation::Owner,
                ..Default::default()
            }
        );
        assert_eq!(
            state.occupants.get(&jid!("b@prose.org")).unwrap(),
            &Occupant {
                affiliation: RoomAffiliation::Member,
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_set_occupant_chat_state() {
        let mut state = RoomState::default();

        state.insert_occupant(
            &jid!("room@prose.org/a"),
            Some(&bare!("a@prose.org")),
            None,
            &RoomAffiliation::Owner,
        );

        state.set_occupant_compose_state(
            &jid!("room@prose.org/a"),
            &Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 0).unwrap(),
            ComposeState::Composing,
        );

        assert_eq!(
            state
                .occupants
                .get(&jid!("room@prose.org/a"))
                .unwrap()
                .compose_state,
            ComposeState::Composing
        );
        assert_eq!(
            state
                .occupants
                .get(&jid!("room@prose.org/a"))
                .unwrap()
                .compose_state_updated,
            Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_composing_users() {
        let mut state = RoomState::default();

        state.occupants.insert(
            jid!("room@prose.org/a"),
            Occupant {
                jid: Some(bare!("a@prose.org")),
                compose_state: ComposeState::Composing,
                compose_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 30).unwrap(),
                ..Default::default()
            },
        );
        state.occupants.insert(
            jid!("room@prose.org/b"),
            Occupant {
                jid: Some(bare!("b@prose.org")),
                compose_state: ComposeState::Idle,
                compose_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 30).unwrap(),
                ..Default::default()
            },
        );
        state.occupants.insert(
            jid!("room@prose.org/c"),
            Occupant {
                jid: Some(bare!("c@prose.org")),
                name: Some("Jonathan Doe".to_string()),
                compose_state: ComposeState::Composing,
                compose_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 20).unwrap(),
                ..Default::default()
            },
        );
        state.occupants.insert(
            jid!("room@prose.org/d"),
            Occupant {
                jid: Some(bare!("d@prose.org")),
                compose_state: ComposeState::Composing,
                compose_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 10).unwrap(),
                ..Default::default()
            },
        );

        assert_eq!(
            state.composing_users(Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 10).unwrap()),
            vec![
                UserBasicInfo {
                    name: "Jonathan Doe".to_string(),
                    jid: bare!("c@prose.org")
                },
                UserBasicInfo {
                    name: "A".to_string(),
                    jid: bare!("a@prose.org")
                },
            ]
        );
    }
}
