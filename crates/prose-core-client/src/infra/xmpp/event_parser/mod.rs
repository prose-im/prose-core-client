// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};
use jid::Jid;
use minidom::Element;
use tracing::info;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::roster::Subscription;

use message::parse_message;
use prose_xmpp::{
    client::Event as XMPPClientEvent, mods::block_list::Event as XMPPBlockListEvent,
    mods::caps::Event as XMPPCapsEvent, mods::chat::Event as XMPPChatEvent,
    mods::muc::Event as XMPPMUCEvent, mods::ping::Event as XMPPPingEvent,
    mods::profile::Event as XMPPProfileEvent, mods::roster::Event as XMPPRosterEvent,
    mods::status::Event as XMPPStatusEvent, Event,
};

use crate::app::event_handlers::{
    BlockListEvent, BlockListEventType, ConnectionEvent, ContactListEvent, ContactListEventType,
    MessageEvent, MessageEventType, OccupantEvent, RequestEvent, RequestEventType, RoomEvent,
    RoomEventType, ServerEvent, SyncedRoomSettingsEvent, UserDeviceEvent, UserInfoEvent,
    UserInfoEventType, UserResourceEvent, UserResourceEventType, UserStatusEvent,
    UserStatusEventType,
};
use crate::app::event_handlers::{SidebarBookmarkEvent, XMPPEvent};
use crate::domain::contacts::models::PresenceSubscription;
use crate::domain::rooms::models::ComposeState;
use crate::domain::shared::models::{CapabilitiesId, MucId, RequestId, SenderId, UserEndpointId};
use crate::dtos::{UserId, UserResourceId};
use crate::infra::xmpp::event_parser::presence::parse_presence;
use crate::infra::xmpp::event_parser::pubsub::parse_pubsub_event;

mod message;
mod presence;
mod pubsub;

pub fn parse_xmpp_event(event: XMPPEvent) -> Result<Vec<ServerEvent>> {
    let mut ctx = Context::default();

    match event {
        Event::BlockList(event) => parse_block_list_event(&mut ctx, event)?,
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
        Event::PubSub(event) => parse_pubsub_event(&mut ctx, event)?,
        Event::Roster(event) => parse_roster_event(&mut ctx, event)?,
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
        XMPPChatEvent::Carbon(carbon) => ctx.push_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Sync(carbon),
        })),
        XMPPChatEvent::Sent(message) => ctx.push_event(ServerEvent::Message(MessageEvent {
            r#type: MessageEventType::Sent(message),
        })),
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
        } => ctx.push_event(UserInfoEvent {
            user_id: UserId::from(from.into_bare()),
            r#type: UserInfoEventType::StatusChanged {
                status: user_activity.map(TryInto::try_into).transpose()?,
            },
        }),
    };
    Ok(())
}

fn parse_block_list_event(ctx: &mut Context, event: XMPPBlockListEvent) -> Result<()> {
    // We're converting the JIDs into UserIds (BareJids) since we only support blocking
    // BareJids for now.
    match event {
        XMPPBlockListEvent::UserBlocked { jid } => ctx.push_event(BlockListEvent {
            r#type: BlockListEventType::UserBlocked {
                user_id: UserId::from(jid.into_bare()),
            },
        }),
        XMPPBlockListEvent::UserUnblocked { jid } => ctx.push_event(BlockListEvent {
            r#type: BlockListEventType::UserUnblocked {
                user_id: UserId::from(jid.into_bare()),
            },
        }),
        XMPPBlockListEvent::BlockListCleared => ctx.push_event(BlockListEvent {
            r#type: BlockListEventType::BlockListCleared,
        }),
    }

    Ok(())
}

fn parse_muc_event(ctx: &mut Context, event: XMPPMUCEvent) -> Result<()> {
    match event {
        XMPPMUCEvent::DirectInvite { from, invite } => {
            let Jid::Full(from) = from else {
                bail!("Expected FullJid in direct invite")
            };

            ctx.push_event(RoomEvent {
                room_id: MucId::from(invite.jid),
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
                room_id: MucId::from(from),
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
                bail!("Expected FullJid in caps element. Found '{from}' instead.")
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
        XMPPClientEvent::PingTimer => ctx.push_event(ConnectionEvent::PingTimer),
    }

    Ok(())
}

fn parse_roster_event(ctx: &mut Context, event: XMPPRosterEvent) -> Result<()> {
    match event {
        XMPPRosterEvent::PresenceSubscriptionRequest { from } => ctx.push_event(ContactListEvent {
            contact_id: UserId::from(from),
            r#type: ContactListEventType::PresenceSubscriptionRequested,
        }),
        XMPPRosterEvent::RosterItemChanged { item } => {
            let event_type = match &item.subscription {
                Subscription::Remove => ContactListEventType::ContactRemoved,
                _ => ContactListEventType::ContactAddedOrPresenceSubscriptionUpdated {
                    subscription: PresenceSubscription::from(&item),
                },
            };

            ctx.push_event(ContactListEvent {
                contact_id: UserId::from(item.jid),
                r#type: event_type,
            })
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
impl From<SidebarBookmarkEvent> for ServerEvent {
    fn from(value: SidebarBookmarkEvent) -> Self {
        Self::SidebarBookmark(value)
    }
}
impl From<ContactListEvent> for ServerEvent {
    fn from(value: ContactListEvent) -> Self {
        Self::ContactList(value)
    }
}
impl From<BlockListEvent> for ServerEvent {
    fn from(value: BlockListEvent) -> Self {
        Self::BlockList(value)
    }
}
impl From<UserDeviceEvent> for ServerEvent {
    fn from(value: UserDeviceEvent) -> Self {
        Self::UserDevice(value)
    }
}
impl From<SyncedRoomSettingsEvent> for ServerEvent {
    fn from(value: SyncedRoomSettingsEvent) -> Self {
        Self::SyncedRoomSettings(value)
    }
}
