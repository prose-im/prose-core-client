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

    /// Returns all composing users that started composing after `started_after`.
    pub fn composing_users(&self, started_after: DateTime<Utc>) -> Vec<BareJid> {
        self.occupants
            .values()
            .filter_map(|occupant| {
                if occupant.chat_state != ChatState::Composing
                    || occupant.chat_state_updated < started_after
                {
                    return None;
                }
                occupant.jid.clone()
            })
            .collect()
    }
}
