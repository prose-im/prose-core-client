// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::Jid;
use xmpp_parsers::muc::user::Status;
use xmpp_parsers::presence;
use xmpp_parsers::presence::Presence;

use prose_xmpp::ns;
use prose_xmpp::stanza::muc::MucUser;

use crate::domain::shared::models::{
    AnonOccupantId, OccupantEvent, OccupantEventType, OccupantId, RoomEvent, RoomEventType,
    UserEndpointId, UserStatusEvent, UserStatusEventType,
};
use crate::dtos::{Availability, RoomId, UserId, UserResourceId};
use crate::infra::xmpp::type_conversions::event_parser::{
    missing_attribute, missing_element, Context,
};

pub fn parse_presence(ctx: &mut Context, presence: Presence) -> Result<()> {
    let Some(from) = presence.from.clone() else {
        return missing_attribute(ctx, "from", presence);
    };

    let availability = Availability::from((
        (presence.type_ != presence::Type::None).then_some(presence.type_.clone()),
        presence.show.clone(),
    ));

    if let Some(muc_user) = presence
        .payloads
        .iter()
        .find(|p| p.is("x", ns::MUC_USER))
        .cloned()
    {
        return parse_muc_presence(ctx, from, availability, presence, muc_user.try_into()?);
    }

    let user_id = match from {
        Jid::Bare(jid) => UserId::from(jid).into(),
        Jid::Full(jid) => UserResourceId::from(jid).into(),
    };

    ctx.push_event(UserStatusEvent {
        user_id,
        r#type: UserStatusEventType::AvailabilityChanged { availability },
    });

    Ok(())
}

fn parse_muc_presence(
    ctx: &mut Context,
    from: Jid,
    availability: Availability,
    presence: Presence,
    mut muc_user: MucUser,
) -> Result<()> {
    let Jid::Full(from) = from else {
        bail!("Expected FullJid in MUC presence.")
    };

    let room = RoomId::from(from.to_bare());

    let Some(item) = muc_user.items.first() else {
        return missing_element(ctx, "item", muc_user);
    };

    let is_self_presence = muc_user.status.contains(&Status::SelfPresence);

    if let Some(destroy) = muc_user.destroy.take() {
        ctx.push_event(RoomEvent {
            room_id: room,
            r#type: RoomEventType::Destroyed {
                replacement: destroy.jid.map(RoomId::from),
            },
        });
        return Ok(());
    }

    let occupant_id = OccupantId::from(from);
    let anon_occupant_id = presence
        .payloads
        .iter()
        .find(|p| p.is("occupant-id", ns::OCCUPANT_ID))
        .and_then(|e| e.attr("id"))
        .map(|id| AnonOccupantId::from(id.to_string()));
    let real_id = item.jid.clone().map(|jid| UserId::from(jid.into_bare()));

    ctx.push_event(UserStatusEvent {
        user_id: UserEndpointId::Occupant(occupant_id.clone()),
        r#type: UserStatusEventType::AvailabilityChanged {
            availability: availability.clone(),
        },
    });

    if availability == Availability::Unavailable {
        if muc_user
            .status
            .iter()
            .find(|s| match s {
                Status::Banned
                | Status::Kicked
                | Status::RemovalFromRoom
                | Status::ConfigMembersOnly => true,
                _ => false,
            })
            .is_some()
        {
            ctx.push_event(OccupantEvent {
                occupant_id,
                anon_occupant_id,
                real_id,
                is_self: is_self_presence,
                r#type: OccupantEventType::PermanentlyRemoved,
            });
            return Ok(());
        }

        if muc_user
            .status
            .iter()
            .find(|s| match s {
                Status::ServiceShutdown | Status::ServiceErrorKick => true,
                _ => false,
            })
            .is_some()
        {
            ctx.push_event(OccupantEvent {
                occupant_id,
                anon_occupant_id,
                real_id,
                is_self: is_self_presence,
                r#type: OccupantEventType::DisconnectedByServer,
            });
            return Ok(());
        }
    }

    // If the user is unavailable and was not banned/room destroyed/forcefully removed then there
    // is no point in sending an AffiliationChanged event, since the affiliation did not change.
    if availability == Availability::Unavailable {
        return Ok(());
    }

    ctx.push_event(OccupantEvent {
        occupant_id,
        anon_occupant_id,
        real_id,
        r#type: OccupantEventType::AffiliationChanged {
            affiliation: item.affiliation.clone().into(),
        },
        is_self: is_self_presence,
    });

    Ok(())
}
