// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::Jid;
use minidom::Element;
use tracing::info;
use xmpp_parsers::message::MessageType;

use message::parse_message;
use prose_xmpp::{
    client::Event as XMPPClientEvent, mods::caps::Event as XMPPCapsEvent,
    mods::chat::Event as XMPPChatEvent, mods::muc::Event as XMPPMUCEvent,
    mods::ping::Event as XMPPPingEvent, mods::profile::Event as XMPPProfileEvent,
    mods::status::Event as XMPPStatusEvent, Event,
};

use crate::app::event_handlers::XMPPEvent;
use crate::app::event_handlers::{
    ConnectionEvent, MessageEvent, OccupantEvent, RequestEvent, RequestEventType, RoomEvent,
    RoomEventType, ServerEvent, UserInfoEvent, UserInfoEventType, UserResourceEvent,
    UserResourceEventType, UserStatusEvent, UserStatusEventType,
};
use crate::domain::rooms::models::ComposeState;
use crate::domain::shared::models::{CapabilitiesId, RequestId, SenderId, UserEndpointId};
use crate::dtos::{RoomId, UserId, UserResourceId};
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
        Event::Caps(event) => parse_caps_event(&mut ctx, event)?,
        Event::Chat(event) => parse_chat_event(&mut ctx, event)?,
        Event::Client(event) => parse_client_event(&mut ctx, event)?,
        Event::MUC(event) => parse_muc_event(&mut ctx, event)?,
        Event::Ping(event) => parse_ping_event(&mut ctx, event)?,
        Event::Profile(event) => parse_profile_event(&mut ctx, event)?,
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
    pub fn push_event(&mut self, event: impl Into<ServerEvent>) {
        self.events.push(event.into())
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
            message_type,
        } => {
            let Jid::Full(from) = from else {
                bail!("Expected FullJid in ChatState")
            };

            let user_id = match message_type {
                MessageType::Groupchat => UserEndpointId::Occupant(from.into()),
                _ => UserEndpointId::UserResource(from.into()),
            };

            ctx.push_event(UserStatusEvent {
                user_id,
                r#type: UserStatusEventType::ComposeStateChanged {
                    state: ComposeState::from(chat_state.clone()),
                },
            })
        }
    };
    Ok(())
}

fn parse_status_event(ctx: &mut Context, event: XMPPStatusEvent) -> Result<()> {
    match event {
        XMPPStatusEvent::Presence(presence) => parse_presence(ctx, presence)?,
        XMPPStatusEvent::UserActivity {
            from,
            user_activity,
        } => {
            ctx.push_event(UserInfoEvent {
                user_id: UserId::from(from.into_bare()),
                r#type: UserInfoEventType::StatusChanged {
                    status: user_activity.map(TryInto::try_into).transpose()?,
                },
            })
        }
    };
    Ok(())
}

fn parse_muc_event(ctx: &mut Context, event: XMPPMUCEvent) -> Result<()> {
    match event {
        XMPPMUCEvent::DirectInvite { from, invite } => {
            let Jid::Full(from) = from else {
                bail!("Expected FullJid in direct invite")
            };

            ctx.push_event(RoomEvent {
                room_id: RoomId::from(invite.jid),
                r#type: RoomEventType::ReceivedInvitation {
                    sender: UserResourceId::from(from),
                    password: invite.password,
                },
            })
        }
        XMPPMUCEvent::MediatedInvite { from, invite } => {
            let Jid::Bare(from) = from else {
                bail!("Expected BareJid for room in mediated invite")
            };

            let Some(embedded_invite) = invite.invites.first() else {
                bail!("Expected MediatedInvite to contain at least one embedded invite.")
            };

            let Some(Jid::Full(sender_jid)) = &embedded_invite.from else {
                bail!("Expected FullJid in embedded invite of MediatedInvite.")
            };

            ctx.push_event(RoomEvent {
                room_id: RoomId::from(from),
                r#type: RoomEventType::ReceivedInvitation {
                    sender: UserResourceId::from(sender_jid.clone()),
                    password: invite.password,
                },
            })
        }
    }

    Ok(())
}

fn parse_caps_event(ctx: &mut Context, event: XMPPCapsEvent) -> Result<()> {
    match event {
        XMPPCapsEvent::DiscoInfoQuery { from, id, node } => {
            let Some(node) = node else {
                bail!("Missing node in disco info query")
            };

            ctx.push_event(RequestEvent {
                sender_id: SenderId::from(from),
                request_id: RequestId::from(id),
                r#type: RequestEventType::Capabilities {
                    id: CapabilitiesId::from(node),
                },
            })
        }
        XMPPCapsEvent::Caps { from, caps } => {
            let Jid::Full(from) = from else {
                bail!("Expected FullJid in caps element")
            };

            ctx.push_event(UserResourceEvent {
                user_id: UserResourceId::from(from),
                r#type: UserResourceEventType::CapabilitiesChanged {
                    id: CapabilitiesId::from(format!("{}#{}", caps.node, caps.hash.to_base64())),
                },
            })
        }
    }

    Ok(())
}

fn parse_profile_event(ctx: &mut Context, event: XMPPProfileEvent) -> Result<()> {
    match event {
        XMPPProfileEvent::Vcard { from, vcard } => ctx.push_event(UserInfoEvent {
            user_id: UserId::from(from.into_bare()),
            r#type: UserInfoEventType::ProfileChanged {
                profile: vcard.try_into()?,
            },
        }),
        XMPPProfileEvent::AvatarMetadata { from, metadata } => {
            let Some(info) = metadata.infos.first() else {
                return missing_element(ctx, "info", metadata);
            };

            ctx.push_event(UserInfoEvent {
                user_id: UserId::from(from.into_bare()),
                r#type: UserInfoEventType::AvatarChanged {
                    metadata: info.clone().into(),
                },
            })
        }
        XMPPProfileEvent::EntityTimeQuery { from, id } => ctx.push_event(RequestEvent {
            sender_id: SenderId::from(from),
            request_id: RequestId::from(id),
            r#type: RequestEventType::LocalTime,
        }),
        XMPPProfileEvent::SoftwareVersionQuery { from, id } => ctx.push_event(RequestEvent {
            sender_id: SenderId::from(from),
            request_id: RequestId::from(id),
            r#type: RequestEventType::SoftwareVersion,
        }),
        XMPPProfileEvent::LastActivityQuery { from, id } => ctx.push_event(RequestEvent {
            sender_id: SenderId::from(from),
            request_id: RequestId::from(id),
            r#type: RequestEventType::LastActivity,
        }),
    }

    Ok(())
}

fn parse_ping_event(ctx: &mut Context, event: XMPPPingEvent) -> Result<()> {
    match event {
        XMPPPingEvent::Ping { from, id } => ctx.push_event(RequestEvent {
            sender_id: SenderId::from(from),
            request_id: RequestId::from(id),
            r#type: RequestEventType::Ping,
        }),
    }

    Ok(())
}

fn parse_client_event(ctx: &mut Context, event: XMPPClientEvent) -> Result<()> {
    match event {
        XMPPClientEvent::Connected => ctx.push_event(ConnectionEvent::Connected),
        XMPPClientEvent::Disconnected { error } => {
            ctx.push_event(ConnectionEvent::Disconnected { error })
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

impl From<ConnectionEvent> for ServerEvent {
    fn from(value: ConnectionEvent) -> Self {
        Self::Connection(value)
    }
}
impl From<UserStatusEvent> for ServerEvent {
    fn from(value: UserStatusEvent) -> Self {
        Self::UserStatus(value)
    }
}
impl From<UserInfoEvent> for ServerEvent {
    fn from(value: UserInfoEvent) -> Self {
        Self::UserInfo(value)
    }
}
impl From<UserResourceEvent> for ServerEvent {
    fn from(value: UserResourceEvent) -> Self {
        Self::UserResource(value)
    }
}
impl From<RoomEvent> for ServerEvent {
    fn from(value: RoomEvent) -> Self {
        Self::Room(value)
    }
}
impl From<OccupantEvent> for ServerEvent {
    fn from(value: OccupantEvent) -> Self {
        Self::Occupant(value)
    }
}
impl From<RequestEvent> for ServerEvent {
    fn from(value: RequestEvent) -> Self {
        Self::Request(value)
    }
}
impl From<MessageEvent> for ServerEvent {
    fn from(value: MessageEvent) -> Self {
        Self::Message(value)
    }
}
