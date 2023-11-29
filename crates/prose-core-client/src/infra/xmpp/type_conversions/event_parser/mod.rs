// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::Jid;
use minidom::Element;
use tracing::info;

use message::parse_message;
use prose_xmpp::{
    mods::chat::Event as XMPPChatEvent, mods::muc::Event as XMPPMUCEvent,
    mods::status::Event as XMPPStatusEvent, Event,
};

use crate::app::event_handlers::XMPPEvent;
use crate::domain::rooms::models::ComposeState;
use crate::domain::shared::models::{RoomEvent, RoomEventType, ServerEvent};
use crate::dtos::RoomId;
use crate::infra::xmpp::type_conversions::event_parser::presence::parse_presence;

mod message;
mod presence;

pub fn parse_xmpp_event(event: XMPPEvent) -> Result<Vec<ServerEvent>> {
    let mut ctx = Context::default();

    match event {
        Event::Bookmark(_) => {
            // TODO: Handle changed bookmarks?
        }
        Event::Bookmark2(_) => {
            // TODO: Handle changed bookmarks?
        }
        Event::Caps(_) => (),
        Event::Chat(event) => parse_chat_event(&mut ctx, event)?,
        Event::Client(_) => (),
        Event::MUC(event) => parse_muc_event(&mut ctx, event)?,
        Event::Ping(_) => (),
        Event::Profile(_) => (),
        Event::PubSub(_) => (),
        Event::Roster(_) => (),
        Event::Status(event) => parse_status_event(&mut ctx, event)?,
    };

    Ok(ctx.events)
}

#[derive(Debug, Default)]
struct Context {
    events: Vec<ServerEvent>,
}

impl Context {
    pub fn push_event(&mut self, event: ServerEvent) {
        self.events.push(event)
    }
}

fn parse_chat_event(ctx: &mut Context, event: XMPPChatEvent) -> Result<()> {
    match event {
        XMPPChatEvent::Message(message) => parse_message(ctx, message)?,
        XMPPChatEvent::Carbon(_) => (),
        XMPPChatEvent::Sent(_) => (),
        XMPPChatEvent::ChatStateChanged {
            from,
            chat_state,
            message_type: _message_type,
        } => {
            let Jid::Full(from) = from else {
                bail!("Expected FullJid in ChatState")
            };

            ctx.push_event(ServerEvent::Room(RoomEvent {
                room_id: RoomId::from(from.to_bare()),
                r#type: RoomEventType::UserComposeStateChanged {
                    user_id: from.clone(),
                    state: ComposeState::from(chat_state.clone()),
                },
            }))
        }
    };
    Ok(())
}

fn parse_status_event(ctx: &mut Context, event: XMPPStatusEvent) -> Result<()> {
    match event {
        XMPPStatusEvent::Presence(presence) => parse_presence(ctx, presence)?,
        XMPPStatusEvent::UserActivity { .. } => (),
    };
    Ok(())
}

fn parse_muc_event(ctx: &mut Context, event: XMPPMUCEvent) -> Result<()> {
    match event {
        XMPPMUCEvent::DirectInvite {
            from: _from,
            invite,
        } => ctx.push_event(ServerEvent::Room(RoomEvent {
            room_id: RoomId::from(invite.jid),
            r#type: RoomEventType::ReceivedInvite {
                password: invite.password,
            },
        })),
        XMPPMUCEvent::MediatedInvite { from, invite } => {
            ctx.push_event(ServerEvent::Room(RoomEvent {
                room_id: RoomId::from(from.to_bare()),
                r#type: RoomEventType::ReceivedInvite {
                    password: invite.password,
                },
            }))
        }
    }

    Ok(())
}

fn ignore_stanza(_ctx: &mut Context, stanza: impl Into<Element>) -> Result<()> {
    info!("Ignoring stanza {}", String::from(&stanza.into()));
    Ok(())
}

fn missing_attribute(
    _ctx: &mut Context,
    attribute: &str,
    stanza: impl Into<Element>,
) -> Result<()> {
    let element = stanza.into();
    Err(anyhow::format_err!(
        "Missing attribute `{}` in {}. {}",
        attribute,
        element.name(),
        String::from(&element)
    ))
}

fn missing_element(
    _ctx: &mut Context,
    element_name: &str,
    stanza: impl Into<Element>,
) -> Result<()> {
    let element = stanza.into();
    Err(anyhow::format_err!(
        "Missing element `{}` in {}. {}",
        element_name,
        element.name(),
        String::from(&element)
    ))
}
