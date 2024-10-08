// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{anyhow, Result};
use jid::Jid;
use xmpp_parsers::muc::user::Status;
use xmpp_parsers::presence::Presence;

use prose_xmpp::stanza::muc::MucUser;

use crate::app::event_handlers::{
    OccupantEvent, OccupantEventType, RoomEvent, RoomEventType, UserStatusEvent,
    UserStatusEventType,
};
use crate::domain::shared::models::{MucId, OccupantId, UserEndpointId};
use crate::dtos::{Availability, UserId, UserResourceId};
use crate::infra::xmpp::event_parser::{missing_attribute, missing_element, Context};
use crate::infra::xmpp::util::PresenceExt;

pub fn parse_presence(ctx: &mut Context, presence: Presence) -> Result<()> {
    let Some(from) = presence.from.clone() else {
        return missing_attribute(ctx, "from", presence);
    };

    if let Some(muc_user) = presence.muc_user() {
        return parse_muc_presence(ctx, from, presence, muc_user);
    }

    let user_id = UserId::from(from.to_bare());
    let endpoint_id = match from.try_into_full() {
        Ok(full) => UserResourceId::from(full).into(),
        Err(bare) => UserId::from(bare).into(),
    };

    ctx.push_event(UserStatusEvent {
        user_id: endpoint_id,
        r#type: UserStatusEventType::PresenceChanged {
            presence: presence.to_domain_presence(user_id, None),
        },
    });

    Ok(())
}

fn parse_muc_presence(
    ctx: &mut Context,
    from: Jid,
    presence: Presence,
    mut muc_user: MucUser,
) -> Result<()> {
    let from = from
        .try_into_full()
        .map_err(|_| anyhow!("Expected FullJid in MUC presence."))?;
    let availability = presence.availability();
    let occupant_id = OccupantId::from(from);

    let Some(item) = muc_user.items.first() else {
        return missing_element(ctx, "item", muc_user);
    };

    let is_self_presence = muc_user.status.contains(&Status::SelfPresence);

    if let Some(destroy) = muc_user.destroy.take() {
        ctx.push_event(RoomEvent {
            room_id: occupant_id.muc_id(),
            r#type: RoomEventType::Destroyed {
                replacement: destroy.jid.map(MucId::from),
            },
        });
        return Ok(());
    }

    let anon_occupant_id = presence.anon_occupant_id();
    let real_id = item.jid.clone().map(|jid| UserId::from(jid.into_bare()));

    ctx.push_event(UserStatusEvent {
        user_id: UserEndpointId::Occupant(occupant_id.clone()),
        r#type: UserStatusEventType::PresenceChanged {
            presence: presence.to_domain_presence(occupant_id.clone(), real_id.clone()),
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
