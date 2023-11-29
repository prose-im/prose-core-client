// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::Jid;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc::user::Status;

use prose_xmpp::ns;
use prose_xmpp::stanza::muc::MucUser;
use prose_xmpp::stanza::Message;

use crate::domain::shared::models::{RoomEvent, RoomEventType, ServerEvent};
use crate::dtos::RoomId;
use crate::infra::xmpp::type_conversions::event_parser::{
    ignore_stanza, missing_attribute, Context,
};

pub fn parse_message(ctx: &mut Context, message: Message) -> Result<()> {
    let Some(from) = message.from.clone() else {
        return missing_attribute(ctx, "from", message);
    };

    match message.type_ {
        MessageType::Groupchat => parse_group_chat_message(ctx, from, message)?,
        MessageType::Chat | MessageType::Normal => parse_chat_message(ctx, from, message)?,
        MessageType::Headline | MessageType::Error => ignore_stanza(ctx, message)?,
    };
    Ok(())
}

fn parse_group_chat_message(ctx: &mut Context, from: Jid, message: Message) -> Result<()> {
    let from = RoomId::from(from.to_bare());

    if let Some(elem) = &message.payloads.iter().find(|p| p.is("x", ns::MUC_USER)) {
        let muc_user = MucUser::try_from((*elem).clone())?;
        if muc_user
            .status
            .iter()
            .find(|s| match *s {
                Status::ConfigNonPrivacyRelated
                | Status::ConfigShowsUnavailableMembers
                | Status::ConfigHidesUnavailableMembers
                | Status::ConfigMembersOnly
                | Status::ConfigRoomLoggingDisabled
                | Status::ConfigRoomLoggingEnabled
                | Status::ConfigRoomNonAnonymous
                | Status::ConfigRoomSemiAnonymous => true,
                _ => false,
            })
            .is_some()
        {
            ctx.push_event(ServerEvent::Room(RoomEvent {
                room: from.clone(),
                r#type: RoomEventType::RoomConfigChanged,
            }))
        }
    }

    if let Some(subject) = message.subject() {
        ctx.push_event(ServerEvent::Room(RoomEvent {
            room: from,
            r#type: RoomEventType::RoomTopicChanged {
                new_topic: (!subject.is_empty()).then_some(subject.to_string()),
            },
        }));
        return Ok(());
    }

    Ok(())
}

fn parse_chat_message(_ctx: &mut Context, _from: Jid, _message: Message) -> Result<()> {
    Ok(())
}
