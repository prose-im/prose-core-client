// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use jid::{BareJid, Jid};
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::muc::user::Affiliation;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct RoomState {
    /// The room's subject.
    pub subject: Option<String>,
    /// The occupants of the room. The key is either the user's FullJid in a MUC room or the user's
    /// BareJid in direct message room.
    pub occupants: HashMap<Jid, Occupant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Occupant {
    /// The real JID of the occupant. Only available in non-anonymous rooms.
    pub jid: Option<BareJid>,
    pub affiliation: Affiliation,
    pub occupant_id: Option<String>,
    pub chat_state: ChatState,
    pub chat_state_updated: DateTime<Utc>,
}

impl Default for Occupant {
    fn default() -> Self {
        Self {
            jid: None,
            affiliation: Default::default(),
            occupant_id: None,
            chat_state: ChatState::Gone,
            chat_state_updated: Default::default(),
        }
    }
}

impl RoomState {
    pub fn insert_occupant(
        &mut self,
        jid: &Jid,
        real_jid: Option<&BareJid>,
        affiliation: &Affiliation,
    ) {
        let occupant = self.occupants.entry(jid.clone()).or_default();
        occupant.jid = real_jid.cloned();
        occupant.affiliation = affiliation.clone();
    }

    pub fn set_occupant_chat_state(
        &mut self,
        occupant_jid: &Jid,
        timestamp: &DateTime<Utc>,
        chat_state: ChatState,
    ) {
        self.occupants
            .entry(occupant_jid.clone())
            .and_modify(|occupant| {
                occupant.chat_state = chat_state;
                occupant.chat_state_updated = timestamp.clone()
            });
    }

    /// Returns the real JIDs of all composing users that started composing after `started_after`.
    /// If we don't have a real JID for a composing user they are excluded from the list.
    pub fn composing_users(&self, started_after: DateTime<Utc>) -> Vec<BareJid> {
        let mut composing_occupants = self
            .occupants
            .values()
            .filter_map(|occupant| {
                if occupant.chat_state != ChatState::Composing
                    || occupant.chat_state_updated <= started_after
                    || occupant.jid.is_none()
                {
                    return None;
                }
                Some(occupant.clone())
            })
            .collect::<Vec<_>>();
        composing_occupants.sort_by_key(|o| o.chat_state_updated);
        composing_occupants
            .into_iter()
            .filter_map(|occupant| occupant.jid)
            .collect()
    }

    pub fn real_jid_for_occupant(&self, occupant_jid: &Jid) -> Option<BareJid> {
        self.occupants.get(occupant_jid).and_then(|o| o.jid.clone())
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
            &Affiliation::Owner,
        );
        state.insert_occupant(&jid!("b@prose.org"), None, &Affiliation::Member);

        assert_eq!(state.occupants.len(), 2);
        assert_eq!(
            state.occupants.get(&jid!("room@prose.org/a")).unwrap(),
            &Occupant {
                jid: Some(bare!("a@prose.org")),
                affiliation: Affiliation::Owner,
                occupant_id: None,
                chat_state: ChatState::Gone,
                chat_state_updated: Default::default(),
            }
        );
        assert_eq!(
            state.occupants.get(&jid!("b@prose.org")).unwrap(),
            &Occupant {
                jid: None,
                affiliation: Affiliation::Member,
                occupant_id: None,
                chat_state: ChatState::Gone,
                chat_state_updated: Default::default(),
            }
        );
    }

    #[test]
    fn test_set_occupant_chat_state() {
        let mut state = RoomState::default();

        state.insert_occupant(
            &jid!("room@prose.org/a"),
            Some(&bare!("a@prose.org")),
            &Affiliation::Owner,
        );

        state.set_occupant_chat_state(
            &jid!("room@prose.org/a"),
            &Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 0).unwrap(),
            ChatState::Composing,
        );

        assert_eq!(
            state
                .occupants
                .get(&jid!("room@prose.org/a"))
                .unwrap()
                .chat_state,
            ChatState::Composing
        );
        assert_eq!(
            state
                .occupants
                .get(&jid!("room@prose.org/a"))
                .unwrap()
                .chat_state_updated,
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
                chat_state: ChatState::Composing,
                chat_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 30).unwrap(),
                ..Default::default()
            },
        );
        state.occupants.insert(
            jid!("room@prose.org/b"),
            Occupant {
                jid: Some(bare!("b@prose.org")),
                chat_state: ChatState::Active,
                chat_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 30).unwrap(),
                ..Default::default()
            },
        );
        state.occupants.insert(
            jid!("room@prose.org/c"),
            Occupant {
                jid: Some(bare!("c@prose.org")),
                chat_state: ChatState::Composing,
                chat_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 20).unwrap(),
                ..Default::default()
            },
        );
        state.occupants.insert(
            jid!("room@prose.org/d"),
            Occupant {
                jid: Some(bare!("d@prose.org")),
                chat_state: ChatState::Composing,
                chat_state_updated: Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 10).unwrap(),
                ..Default::default()
            },
        );

        assert_eq!(
            state.composing_users(Utc.with_ymd_and_hms(2023, 01, 03, 0, 0, 10).unwrap()),
            vec![bare!("c@prose.org"), bare!("a@prose.org")]
        );
    }
}
