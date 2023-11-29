// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::Jid;
use xmpp_parsers::muc::user::Status;
use xmpp_parsers::presence;
use xmpp_parsers::presence::Presence;

use crate::domain::rooms::models::RoomAffiliation;
use prose_xmpp::ns;
use prose_xmpp::stanza::muc::MucUser;

use crate::domain::shared::models::{RoomEvent, RoomEventType, RoomUserInfo, ServerEvent};
use crate::dtos::{Availability, RoomJid};
use crate::infra::xmpp::type_conversions::event_parser::{
    missing_attribute, missing_element, Context,
};

pub fn parse_presence(ctx: &mut Context, presence: Presence) -> Result<()> {
    if let Some(muc_user) = presence
        .payloads
        .iter()
        .find(|p| p.is("x", ns::MUC_USER))
        .cloned()
    {
        return parse_muc_presence(ctx, presence, muc_user.try_into()?);
    }

    Ok(())
}

fn parse_muc_presence(ctx: &mut Context, presence: Presence, mut muc_user: MucUser) -> Result<()> {
    let Some(Jid::Full(from)) = presence.from else {
        return missing_attribute(ctx, "from", presence);
    };

    let room = RoomJid::from(from.to_bare());

    let Some(item) = muc_user.items.first() else {
        return missing_element(ctx, "item", muc_user);
    };

    let is_self_presence = muc_user.status.contains(&Status::SelfPresence);

    let user = RoomUserInfo {
        jid: from,
        real_jid: item.jid.clone(),
        affiliation: RoomAffiliation::from(item.affiliation.clone()),
        availability: Availability::from((
            (presence.type_ != presence::Type::None).then_some(presence.type_),
            presence.show,
        )),
        is_self: is_self_presence,
    };

    if let Some(destroy) = muc_user.destroy.take() {
        ctx.push_event(ServerEvent::Room(RoomEvent {
            room,
            r#type: RoomEventType::RoomWasDestroyed {
                alternate_room: destroy.jid.map(RoomJid::from),
            },
        }));
        return Ok(());
    }

    if user.availability == Availability::Unavailable {
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
            ctx.push_event(ServerEvent::Room(RoomEvent {
                room,
                r#type: RoomEventType::UserWasPermanentlyRemoved { user },
            }));
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
            ctx.push_event(ServerEvent::Room(RoomEvent {
                room,
                r#type: RoomEventType::UserWasDisconnectedByServer { user },
            }));
            return Ok(());
        }
    }

    ctx.push_event(ServerEvent::Room(RoomEvent {
        room,
        r#type: RoomEventType::UserAvailabilityOrMembershipChanged { user },
    }));

    Ok(())
}
