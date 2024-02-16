// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::Jid;
use prose_xmpp::mods::muc::RoomOccupancy;
use prose_xmpp::stanza::muc::MucUser;
use tracing::error;
use xmpp_parsers::muc::user::Status;
use xmpp_parsers::presence::Presence;

use crate::domain::rooms::models::RoomSessionParticipant;
use crate::dtos::{OccupantId, UserId};
use crate::infra::xmpp::util::PresenceExt;

pub trait RoomOccupancyExt {
    fn participants(&self) -> Vec<RoomSessionParticipant>;
}

impl RoomOccupancyExt for RoomOccupancy {
    fn participants(&self) -> Vec<RoomSessionParticipant> {
        self.presences
            .iter()
            .filter_map(|p| match RoomSessionParticipant::try_from(p.clone()) {
                Ok(participant) => Some(participant),
                Err(err) => {
                    error!(
                        "Failed to parse MUC presence '{:?}' in RoomOccupancy. {}",
                        p.from,
                        err.to_string(),
                    );
                    None
                }
            })
            .chain(self_participant(&self.self_presence, &self.user))
            .collect()
    }
}

fn self_participant(presence: &Presence, muc_user: &MucUser) -> Result<RoomSessionParticipant> {
    let Some(Jid::Full(from)) = &presence.from else {
        bail!("Expected FullJid in MUC presence.")
    };

    let Some(item) = muc_user.items.first() else {
        bail!("Missing 'item' element in MUC user");
    };

    Ok(RoomSessionParticipant {
        id: OccupantId::from(from.clone()),
        is_self: muc_user.status.contains(&Status::SelfPresence),
        anon_id: presence.anon_occupant_id(),
        real_id: item.jid.clone().map(|jid| UserId::from(jid.into_bare())),
        affiliation: item.affiliation.clone().into(),
        availability: presence.availability(),
    })
}
