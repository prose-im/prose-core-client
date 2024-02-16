// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::iter;
use tracing::error;

use prose_xmpp::mods::muc::RoomOccupancy;

use crate::domain::rooms::models::RoomSessionParticipant;

pub trait RoomOccupancyExt {
    fn participants(&self) -> Vec<RoomSessionParticipant>;
}

impl RoomOccupancyExt for RoomOccupancy {
    fn participants(&self) -> Vec<RoomSessionParticipant> {
        let mut self_presence = self.self_presence.clone();
        self_presence.payloads.push(self.user.clone().into());

        iter::once(&self_presence)
            .chain(self.presences.iter())
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
            .collect()
    }
}
