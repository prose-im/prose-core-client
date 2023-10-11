// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;

use anyhow::Result;
use jid::{BareJid, FullJid, Jid};
use minidom::Element;
use prose_wasm_utils::SendUnlessWasm;
use xmpp_parsers::data_forms::{DataForm, DataFormType};
use xmpp_parsers::disco::{DiscoItemsQuery, DiscoItemsResult};
use xmpp_parsers::iq::Iq;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc::user::{Affiliation, Status};
use xmpp_parsers::muc::MucUser;
use xmpp_parsers::presence;
use xmpp_parsers::presence::Presence;
use xmpp_parsers::stanza_error::StanzaError;

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::Module;
use crate::ns;
use crate::stanza::muc::mediated_invite::MediatedInvite;
use crate::stanza::muc::query::{Destroy, Role};
use crate::stanza::muc::{DirectInvite, Query};
use crate::stanza::{muc, Message};
use crate::util::{ElementReducerPoll, RequestError, RequestFuture, XMPPElement};

/// XEP-0045: Multi-User Chat
/// https://xmpp.org/extensions/xep-0045.html#disco-rooms
#[derive(Default, Clone)]
pub struct MUC {
    ctx: ModuleContext,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Room {
    pub jid: Jid,
    pub name: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RoomOccupancy {
    pub user: MucUser,
    pub self_presence: Presence,
    pub presences: Vec<Presence>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoomConfigResponse {
    Submit(DataForm),
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// XEP-0249: Direct MUC Invitations
    DirectInvite { from: Jid, invite: DirectInvite },
    /// https://xmpp.org/extensions/xep-0045.html#invite-mediated
    MediatedInvite { from: Jid, invite: MediatedInvite },
}

impl Module for MUC {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }

    fn handle_message_stanza(&self, stanza: &Message) -> Result<()> {
        let Some(from) = &stanza.from else {
            return Ok(());
        };

        if let Some(direct_invite) = &stanza.direct_invite {
            self.ctx
                .schedule_event(ClientEvent::MUC(Event::DirectInvite {
                    from: from.clone(),
                    invite: direct_invite.clone(),
                }));
        };

        if let Some(mediated_invite) = &stanza.mediated_invite {
            // Ignore empty invites.
            if !mediated_invite.invites.is_empty() {
                self.ctx
                    .schedule_event(ClientEvent::MUC(Event::MediatedInvite {
                        from: from.clone(),
                        invite: mediated_invite.clone(),
                    }));
            }
        };

        Ok(())
    }
}

impl MUC {
    /// Loads public rooms in a MUC service.
    /// https://xmpp.org/extensions/xep-0045.html#disco-rooms
    pub async fn load_public_rooms(&self, service: &BareJid) -> Result<Vec<Room>> {
        let response = self
            .ctx
            .send_iq(
                Iq::from_get(self.ctx.generate_id(), DiscoItemsQuery { node: None })
                    .with_to(Jid::Bare(service.clone())),
            )
            .await?
            .ok_or(RequestError::UnexpectedResponse)?;

        let items = DiscoItemsResult::try_from(response)?;

        let rooms = items
            .items
            .into_iter()
            .map(|item| Room {
                jid: item.jid,
                name: item.name,
            })
            .collect();

        Ok(rooms)
    }

    /// Enters a room.
    /// https://xmpp.org/extensions/xep-0045.html#enter
    pub async fn enter_room(
        &self,
        room_jid: &FullJid,
        password: Option<&str>,
    ) -> Result<RoomOccupancy, RequestError> {
        self.send_presence_to_room(&room_jid, password).await
    }

    /// Creates an instant room or joins an existing room with the same JID.
    /// https://xmpp.org/extensions/xep-0045.html#createroom-instant
    pub async fn create_instant_room(
        &self,
        room_jid: &FullJid,
    ) -> Result<RoomOccupancy, RequestError> {
        // https://xmpp.org/extensions/xep-0045.html#createroom
        let occupancy = self.send_presence_to_room(&room_jid, None).await?;

        // If the room existed already we don't need to proceed…
        if !occupancy.user.status.contains(&Status::RoomHasBeenCreated) {
            return Ok(occupancy);
        }

        let iq = Iq::from_set(
            self.ctx.generate_id(),
            muc::Query {
                role: muc::query::Role::Owner,
                payloads: vec![Element::builder("x", ns::DATA_FORMS)
                    .attr("type", "submit")
                    .build()],
            },
        )
        .with_to(room_jid.to_bare().into());

        self.ctx.send_iq(iq).await?;
        Ok(occupancy)
    }

    /// Creates a reserved room or joins an existing room with the same JID. Invokes `handler` to
    /// perform the configuration of the reserved room.
    /// https://xmpp.org/extensions/xep-0045.html#createroom-reserved
    pub async fn create_reserved_room<T>(
        &self,
        room_jid: &FullJid,
        handler: impl FnOnce(DataForm) -> T,
    ) -> Result<RoomOccupancy, RequestError>
    where
        T: Future<Output = Result<RoomConfigResponse>> + SendUnlessWasm + 'static,
    {
        // https://xmpp.org/extensions/xep-0045.html#createroom
        let occupancy = self.send_presence_to_room(&room_jid, None).await?;

        // If the room existed already we don't need to proceed…
        if !occupancy.user.status.contains(&Status::RoomHasBeenCreated) {
            return Ok(occupancy);
        }

        let iq = Iq::from_get(
            self.ctx.generate_id(),
            muc::Query {
                role: muc::query::Role::Owner,
                payloads: vec![],
            },
        )
        .with_to(room_jid.to_bare().into());

        let mut query = muc::Query::try_from(
            self.ctx
                .send_iq(iq)
                .await?
                .ok_or(RequestError::UnexpectedResponse)?,
        )?;

        let form = DataForm::try_from(
            query
                .payloads
                .pop()
                .ok_or(RequestError::UnexpectedResponse)?,
        )?;

        let handler_result = handler(form).await.map_err(|e| RequestError::Generic {
            msg: format!("Handler returned with error {}", e.to_string()),
        })?;

        let response_form = match handler_result {
            RoomConfigResponse::Submit(form) => form,
            RoomConfigResponse::Cancel => DataForm {
                type_: DataFormType::Cancel,
                form_type: None,
                title: None,
                instructions: None,
                fields: vec![],
            },
        };

        let iq = Iq::from_set(
            self.ctx.generate_id(),
            muc::Query {
                role: muc::query::Role::Owner,
                payloads: vec![response_form.into()],
            },
        )
        .with_to(room_jid.to_bare().into());

        self.ctx.send_iq(iq).await?;

        Ok(occupancy)
    }

    /// Destroys a room.
    /// https://xmpp.org/extensions/xep-0045.html#destroyroom
    pub async fn destroy_room(&self, jid: &BareJid) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            Query::new(Role::Owner).with_payload(Destroy {
                jid: None,
                reason: None,
            }),
        )
        .with_to(jid.clone().into());
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    /// Requests the list of users with a given affiliation.
    /// https://xmpp.org/extensions/xep-0045.html#example-129
    pub async fn request_users(
        &self,
        room_jid: &BareJid,
        affiliation: Affiliation,
    ) -> Result<Vec<muc::query::User>, RequestError> {
        let iq = Iq::from_get(
            self.ctx.generate_id(),
            Query {
                role: Role::Admin,
                payloads: vec![xmpp_parsers::muc::user::Item {
                    affiliation,
                    jid: None,
                    nick: None,
                    role: Default::default(),
                    actor: None,
                    continue_: None,
                    reason: None,
                }
                .into()],
            },
        )
        .with_to(room_jid.clone().into());

        let response = self
            .ctx
            .send_iq(iq)
            .await?
            .ok_or(RequestError::UnexpectedResponse)?;

        let query = Query::try_from(response)?;
        let users = query
            .payloads
            .into_iter()
            .filter_map(|payload| {
                if !payload.is("item", ns::MUC_ADMIN) {
                    return None;
                }
                return Some(muc::query::User::try_from(payload));
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(users)
    }

    /// Modifies the affiliation of the given users. Make sure to only send the deltas.
    /// https://xmpp.org/extensions/xep-0045.html#example-183
    pub async fn update_user_affiliations(
        &self,
        room_jid: &BareJid,
        users: impl IntoIterator<Item = (BareJid, Affiliation)>,
    ) -> Result<()> {
        // It seems that we can only send one user at a time, otherwise only the first is used when
        // we're sending all at once…

        for (jid, affiliation) in users.into_iter() {
            let iq = Iq::from_set(
                self.ctx.generate_id(),
                muc::Query {
                    role: Role::Admin,
                    payloads: vec![Element::builder("item", &Role::Admin.to_string())
                        .attr("jid", jid)
                        .attr("affiliation", affiliation)
                        .build()],
                },
            )
            .with_to(room_jid.clone().into());

            self.ctx.send_iq(iq).await?;
        }

        Ok(())
    }

    /// Sends a direct invite to a user.
    /// https://xmpp.org/extensions/xep-0045.html#invite-direct
    pub async fn send_direct_invite(
        &self,
        to: impl Into<Jid>,
        direct_invite: DirectInvite,
    ) -> Result<()> {
        let message = Message {
            to: Some(to.into()),
            direct_invite: Some(direct_invite),
            ..Default::default()
        };
        self.ctx.send_stanza(message)?;
        Ok(())
    }

    /// Sends a mediated invite to a room which in turn forwards it to the invited users.
    /// https://xmpp.org/extensions/xep-0045.html#invite-mediated
    pub async fn send_mediated_invite(
        &self,
        room_jid: &BareJid,
        mediated_invite: MediatedInvite,
    ) -> Result<()> {
        let message = Message {
            to: Some(room_jid.clone().into()),
            mediated_invite: Some(mediated_invite),
            ..Default::default()
        };
        self.ctx.send_stanza(message)?;
        Ok(())
    }

    pub async fn set_room_subject(&self, room_jid: &BareJid, subject: Option<&str>) -> Result<()> {
        let message = Message::new()
            .set_id(self.ctx.generate_id().into())
            .set_type(MessageType::Groupchat)
            .set_to(room_jid.clone())
            .set_subject(subject.unwrap_or_default()); // Send empty string for empty subject
        self.ctx.send_stanza(message)
    }
}

impl MUC {
    async fn send_presence_to_room(
        &self,
        room_jid: &FullJid,
        password: Option<&str>,
    ) -> Result<RoomOccupancy, RequestError> {
        let presence = Presence::new(presence::Type::None)
            .with_to(room_jid.clone())
            .with_payloads(vec![Element::builder("x", ns::MUC)
                .append_all(
                    password.map(|password| Element::builder("password", ns::MUC).append(password)),
                )
                .build()]);

        let (mut self_presence, presences) = self
            .ctx
            .send_stanza_with_future(
                presence,
                RequestFuture::new_join_room_request(room_jid.clone()),
            )
            .await?;

        let payload = self_presence
            .payloads
            .pop()
            .ok_or(RequestError::UnexpectedResponse)?;

        Ok(RoomOccupancy {
            user: MucUser::try_from(payload)?,
            self_presence,
            presences,
        })
    }
}

/// Order of events (https://xmpp.org/extensions/xep-0045.html#order)
///   1. In-room presence from other occupants
///   2. In-room presence from the joining entity itself (so-called "self-presence")
///   3. Room history (if any)
///   4. The room subject
///   5. Live messages, presence updates, new user joins, etc.
///
/// We're running our Future for steps 1 & 2. The remaining steps need to be handled by the
/// Client's event handler.  
struct JoinRoomState {
    pub room_jid: FullJid,
    pub presences: Vec<Presence>,
    pub self_presence: Option<Presence>,
}

impl RequestFuture<JoinRoomState, (Presence, Vec<Presence>)> {
    pub fn new_join_room_request(room_jid: FullJid) -> Self {
        RequestFuture::new(
            JoinRoomState {
                room_jid,
                presences: vec![],
                self_presence: None,
            },
            |state, element| {
                let XMPPElement::Presence(presence) = element else {
                    return Ok(ElementReducerPoll::Pending);
                };

                let Some(Jid::Full(from)) = &presence.from else {
                    return Ok(ElementReducerPoll::Pending);
                };

                // Make sure that the presence is actually sent by our room…
                if from.node() != state.room_jid.node() || from.domain() != state.room_jid.domain()
                {
                    return Ok(ElementReducerPoll::Pending);
                }

                // Is that the self-presence or somebody else's?
                let is_self_presence = from.resource() == state.room_jid.resource();

                // Check if we have an error on our hands (which is addressed at us directly)…
                if presence.type_ == presence::Type::Error && is_self_presence {
                    return if let Some(error_payload) =
                        presence.payloads.iter().find(|p| p.name() == "error")
                    {
                        match StanzaError::try_from(error_payload.clone()) {
                            Ok(err) => Err(RequestError::XMPP { err }),
                            Err(error) => Err(RequestError::Generic {
                                msg: error.to_string(),
                            }),
                        }
                    } else {
                        Err(RequestError::Generic {
                            msg:
                                "Encountered presence of type error with a missing `error` stanza."
                                    .to_string(),
                        })
                    };
                }

                if !is_self_presence {
                    state.presences.push(presence.clone());
                    return Ok(ElementReducerPoll::Pending);
                }

                state.self_presence = Some(presence.clone());
                Ok(ElementReducerPoll::Ready)
            },
            |state| {
                (
                    state
                        .self_presence
                        .expect("Internal error. Missing response in PresenceFutureState."),
                    state.presences,
                )
            },
        )
    }
}
