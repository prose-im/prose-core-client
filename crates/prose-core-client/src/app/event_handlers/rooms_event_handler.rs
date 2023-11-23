// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::{BareJid, Jid};
use tracing::{error, info, warn};
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc::user::{Role, Status};
use xmpp_parsers::presence::Presence;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::{bookmark, bookmark2, chat, muc, status};
use prose_xmpp::stanza::muc::MucUser;
use prose_xmpp::{ns, Event};

use crate::app::deps::{
    DynAppContext, DynClientEventDispatcher, DynConnectedRoomsReadOnlyRepository,
    DynSidebarDomainService, DynTimeProvider, DynUserProfileRepository,
};
use crate::app::event_handlers::{XMPPEvent, XMPPEventHandler};
use crate::client_event::RoomEventType;
use crate::domain::messaging::models::{MessageLike, MessageLikePayload};
use crate::domain::rooms::services::CreateOrEnterRoomRequest;
use crate::domain::shared::models::RoomJid;

#[derive(InjectDependencies)]
pub struct RoomsEventHandler {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
    #[inject]
    time_provider: DynTimeProvider,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl XMPPEventHandler for RoomsEventHandler {
    fn name(&self) -> &'static str {
        "rooms"
    }

    async fn handle_event(&self, event: XMPPEvent) -> Result<Option<XMPPEvent>> {
        match event {
            Event::Status(event) => match event {
                status::Event::Presence(presence) => {
                    self.presence_did_change(presence).await?;
                    Ok(None)
                }
                _ => Ok(Some(Event::Status(event))),
            },
            Event::Chat(event) => match event {
                chat::Event::ChatStateChanged {
                    from,
                    chat_state,
                    message_type,
                } => {
                    self.handle_changed_chat_state(from, chat_state, message_type)
                        .await?;
                    Ok(None)
                }
                _ => Ok(Some(Event::Chat(event))),
            },
            Event::MUC(event) => match event {
                muc::Event::DirectInvite {
                    from: _from,
                    invite,
                } => {
                    self.handle_invite(invite.jid, invite.password).await?;
                    Ok(None)
                }
                muc::Event::MediatedInvite { from, invite } => {
                    self.handle_invite(from.to_bare(), invite.password).await?;
                    Ok(None)
                }
            },
            Event::Bookmark(event) => match event {
                bookmark::Event::BookmarksChanged {
                    bookmarks: _bookmarks,
                } => {
                    // TODO: Handle changed bookmarks
                    Ok(None)
                }
            },
            Event::Bookmark2(event) => match event {
                bookmark2::Event::BookmarksPublished {
                    bookmarks: _bookmarks,
                } => {
                    // TODO: Handle changed bookmarks
                    Ok(None)
                }
                bookmark2::Event::BookmarksRetracted { jids: _jids } => {
                    // TODO: Handle changed bookmarks
                    Ok(None)
                }
            },
            _ => Ok(Some(event)),
        }
    }
}

impl RoomsEventHandler {
    async fn presence_did_change(&self, presence: Presence) -> Result<()> {
        let Some(from) = presence.from else {
            error!(
                "Received presence from unknown user. {}",
                String::from(&minidom::Element::from(presence))
            );
            return Ok(());
        };

        let from = from;
        let bare_from = RoomJid::from(from.to_bare());

        // Ignore presences that were sent by us. We don't have a room for the logged-in user.
        if *bare_from == self.ctx.connected_jid()?.into_bare() {
            return Ok(());
        }

        let Some(room) = self.connected_rooms_repo.get(&bare_from) else {
            warn!(
                "Received presence from user ({}) for which we do not have a room.",
                from
            );
            return Ok(());
        };

        let Some(mut muc_user) = presence
            .payloads
            .into_iter()
            .filter_map(|payload| {
                if !payload.is("x", ns::MUC_USER) {
                    return None;
                }
                MucUser::try_from(payload).ok()
            })
            .take(1)
            .next()
        else {
            return Ok(());
        };

        let is_self_presence = muc_user.status.contains(&Status::SelfPresence);
        if is_self_presence {
            if let Some(destroy) = muc_user.destroy.take() {
                info!(
                    "Room {} has been destroyed. Alternative is {}",
                    room.jid,
                    destroy
                        .jid
                        .as_ref()
                        .map(|j| j.to_string())
                        .as_deref()
                        .unwrap_or("<none>")
                );
                self.sidebar_domain_service
                    .handle_destroyed_room(&room.jid, destroy.jid.map(RoomJid::from))
                    .await?;
                return Ok(());
            }
        }

        let Some(item) = muc_user.items.first() else {
            return Ok(());
        };

        // User has been removed or went offline…
        if item.role == Role::None {
            room.remove_occupant(&from);

            if is_self_presence {
                let was_removed = muc_user
                    .status
                    .iter()
                    .find(|s| match s {
                        Status::Banned
                        | Status::Kicked
                        | Status::RemovalFromRoom
                        | Status::ConfigMembersOnly
                        | Status::ServiceShutdown
                        | Status::ServiceErrorKick => true,
                        _ => false,
                    })
                    .is_some();

                if was_removed {
                    let is_permanent = muc_user
                        .status
                        .iter()
                        .find(|s| match s {
                            Status::Banned
                            | Status::Kicked
                            | Status::RemovalFromRoom
                            | Status::ConfigMembersOnly => true,
                            _ => false,
                        })
                        .is_some();

                    self.sidebar_domain_service
                        .handle_removal_from_room(&room.jid, is_permanent)
                        .await?;
                }
            }
        } else {
            // Let's try to pull out the real jid of our user…
            let (real_jid, name) = {
                if let Some(jid) = &item.jid {
                    let bare_jid = jid.to_bare();
                    let name = self.user_profile_repo.get_display_name(&bare_jid).await?;
                    (Some(bare_jid), name)
                } else {
                    (None, None)
                }
            };

            room.insert_occupant(&from, real_jid.as_ref(), name.as_deref(), &item.affiliation);
        }

        Ok(())
    }

    async fn handle_invite(&self, room_jid: BareJid, password: Option<String>) -> Result<()> {
        info!("Joining room {} after receiving invite…", room_jid);

        self.sidebar_domain_service
            .insert_item_by_creating_or_joining_room(CreateOrEnterRoomRequest::JoinRoom {
                room_jid: RoomJid::from(room_jid),
                password,
            })
            .await?;

        Ok(())
    }

    pub async fn handle_changed_chat_state(
        &self,
        from: Jid,
        chat_state: ChatState,
        message_type: MessageType,
    ) -> Result<()> {
        let bare_from = RoomJid::from(from.to_bare());

        let Some(room) = self.connected_rooms_repo.get(&bare_from) else {
            error!("Received chat state from sender for which we do not have a room.");
            return Ok(());
        };

        let jid = if message_type == MessageType::Groupchat {
            from
        } else {
            Jid::Bare(bare_from.into_inner())
        };
        let now = self.time_provider.now();

        room.set_occupant_chat_state(&jid, &now, chat_state);

        self.client_event_dispatcher
            .dispatch_room_event(room, RoomEventType::ComposingUsersChanged);

        Ok(())
    }
}

impl From<&MessageLike> for RoomEventType {
    fn from(message: &MessageLike) -> Self {
        if let Some(ref target) = message.target {
            if message.payload == MessageLikePayload::Retraction {
                Self::MessagesDeleted {
                    message_ids: vec![target.as_ref().into()],
                }
            } else {
                Self::MessagesUpdated {
                    message_ids: vec![target.as_ref().into()],
                }
            }
        } else {
            Self::MessagesAppended {
                message_ids: vec![message.id.id().as_ref().into()],
            }
        }
    }
}
