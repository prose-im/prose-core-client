// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::Jid;
use tracing::error;
use xmpp_parsers::muc::user::Status;
use xmpp_parsers::presence::Presence;

use prose_xmpp::mods::muc::RoomOccupancy;
use prose_xmpp::stanza::muc::MucUser;

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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use xmpp_parsers::muc::user::{Affiliation, Item, Role};
    use xmpp_parsers::occupant_id::OccupantId as XMPPOccupantId;

    use prose_xmpp::{bare, full};

    use crate::domain::shared::models::AnonOccupantId;
    use crate::dtos::{Availability, RoomAffiliation};
    use crate::{occupant_id, user_id};

    use super::*;

    #[test]
    fn test_try_from_room_occupancy() {
        let occupancy = RoomOccupancy {
            user: MucUser::new()
                .with_item(
                    Item::new(Affiliation::Member, Role::Visitor)
                        .with_jid(full!("me@prose.org/res")),
                )
                .with_status(vec![Status::SelfPresence]),
            self_presence: Presence::new(Default::default())
                .with_from(full!("room@conf.prose.org/me"))
                .with_to(bare!("user@prose.org"))
                .with_payload(XMPPOccupantId {
                    id: "occ_3".to_string(),
                }),
            presences: vec![
                Presence::new(Default::default())
                    .with_from(full!("room@conf.prose.org/user_a"))
                    .with_payload(XMPPOccupantId {
                        id: "occ_1".to_string(),
                    })
                    .with_payload(
                        MucUser::new().with_item(
                            Item::new(Affiliation::Member, Role::Moderator)
                                .with_jid(full!("user_a@prose.org/res")),
                        ),
                    ),
                Presence::new(Default::default())
                    .with_from(full!("room@conf.prose.org/user_b"))
                    .with_payload(XMPPOccupantId {
                        id: "occ_2".to_string(),
                    })
                    .with_payload(
                        MucUser::new().with_item(
                            Item::new(Affiliation::Member, Role::Participant)
                                .with_jid(full!("user_b@prose.org/res")),
                        ),
                    ),
            ],
            subject: Some("Room Subject".to_string()),
            message_history: vec![],
        };

        assert_eq!(
            occupancy.participants(),
            vec![
                RoomSessionParticipant {
                    id: occupant_id!("room@conf.prose.org/user_a"),
                    is_self: false,
                    anon_id: Some(AnonOccupantId::from("occ_1")),
                    real_id: Some(user_id!("user_a@prose.org")),
                    affiliation: RoomAffiliation::Member,
                    availability: Availability::Available,
                },
                RoomSessionParticipant {
                    id: occupant_id!("room@conf.prose.org/user_b"),
                    is_self: false,
                    anon_id: Some(AnonOccupantId::from("occ_2")),
                    real_id: Some(user_id!("user_b@prose.org")),
                    affiliation: RoomAffiliation::Member,
                    availability: Availability::Available,
                },
                RoomSessionParticipant {
                    id: occupant_id!("room@conf.prose.org/me"),
                    is_self: true,
                    anon_id: Some(AnonOccupantId::from("occ_3")),
                    real_id: Some(user_id!("me@prose.org")),
                    affiliation: RoomAffiliation::Member,
                    availability: Availability::Available,
                }
            ]
        );
    }
}
