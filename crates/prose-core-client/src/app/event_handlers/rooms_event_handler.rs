// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use tracing::{error, info};
use xmpp_parsers::muc::MucUser;
use xmpp_parsers::presence::Presence;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::{bookmark, bookmark2, muc, status};
use prose_xmpp::{ns, Event};

use crate::app::deps::{DynConnectedRoomsRepository, DynRoomsDomainService};
use crate::app::event_handlers::{XMPPEvent, XMPPEventHandler};
use crate::client_event::RoomEventType;
use crate::domain::messaging::models::{MessageLike, MessageLikePayload};
use crate::domain::rooms::services::{CreateOrEnterRoomRequest, CreateOrEnterRoomRequestType};

#[derive(InjectDependencies)]
pub(crate) struct RoomsEventHandler {
    #[inject]
    connected_rooms_repo: DynConnectedRoomsRepository,
    #[inject]
    rooms_domain_service: DynRoomsDomainService,
}

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
        let Some(to) = presence.to else {
            error!("Received presence from unknown user.");
            return Ok(());
        };

        let to = to.into_bare();

        let Some(room) = self.connected_rooms_repo.get(&to) else {
            error!("Received presence from user for which we do not have a room.");
            return Ok(());
        };

        let Some(from) = &presence.from else {
            return Ok(());
        };

        let Some(muc_user) = presence
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

        // Let's try to pull out the real jid of our user…
        let Some((jid, affiliation)) = muc_user
            .items
            .into_iter()
            .filter_map(|item| item.jid.map(|jid| (jid, item.affiliation)))
            .take(1)
            .next()
        else {
            return Ok(());
        };

        info!("Received real jid for {}: {}", from, jid);
        room.state
            .write()
            .insert_occupant(from, Some(&jid.into_bare()), &affiliation);

        Ok(())
    }

    async fn handle_invite(&self, room_jid: BareJid, password: Option<String>) -> Result<()> {
        info!("Joining room {} after receiving invite…", room_jid);

        self.rooms_domain_service
            .create_or_join_room(CreateOrEnterRoomRequest {
                r#type: CreateOrEnterRoomRequestType::Join {
                    room_jid,
                    nickname: None,
                    password,
                },
                save_bookmark: true,
                notify_delegate: true,
            })
            .await?;

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
