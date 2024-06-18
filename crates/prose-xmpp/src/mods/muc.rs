// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;

use anyhow::Result;
use jid::{BareJid, FullJid, Jid};
use minidom::Element;
use xmpp_parsers::data_forms::{DataForm, DataFormType};
use xmpp_parsers::disco::{DiscoItemsQuery, DiscoItemsResult};
use xmpp_parsers::iq::Iq;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc::user::{Affiliation, Status};
use xmpp_parsers::presence;
use xmpp_parsers::presence::{Presence, Show};
use xmpp_parsers::stanza_error::StanzaError;

use prose_wasm_utils::SendUnlessWasm;

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::Module;
use crate::ns;
use crate::stanza::muc::mediated_invite::MediatedInvite;
use crate::stanza::muc::query::{Destroy, Role};
use crate::stanza::muc::{DirectInvite, MucUser, Query};
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
    pub subject: Option<String>,
    pub message_history: Vec<Message>,
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

        if stanza.type_ != MessageType::Normal {
            return Ok(());
        }

        if let Some(direct_invite) = stanza.direct_invite() {
            self.ctx
                .schedule_event(ClientEvent::MUC(Event::DirectInvite {
                    from: from.clone(),
                    invite: direct_invite,
                }));
        };

        if let Some(mediated_invite) = stanza.mediated_invite() {
            // Ignore empty invites.
            if !mediated_invite.invites.is_empty() {
                self.ctx
                    .schedule_event(ClientEvent::MUC(Event::MediatedInvite {
                        from: from.clone(),
                        invite: mediated_invite,
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
                Iq::from_get(
                    self.ctx.generate_id(),
                    DiscoItemsQuery {
                        node: None,
                        rsm: None,
                    },
                )
                .with_to(Jid::from(service.clone())),
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
        show: Option<Show>,
        caps: Option<xmpp_parsers::caps::Caps>,
    ) -> Result<RoomOccupancy, RequestError> {
        self.send_presence_to_room(&room_jid, password, show, caps)
            .await
    }

    /// Exits a room.
    /// https://xmpp.org/extensions/xep-0045.html#example-80
    pub async fn exit_room(&self, room_jid: &FullJid) -> Result<(), RequestError> {
        self.ctx
            .send_stanza(Presence::new(presence::Type::Unavailable).with_to(room_jid.clone()))?;
        Ok(())
    }

    /// Creates an instant room or joins an existing room with the same JID.
    /// https://xmpp.org/extensions/xep-0045.html#createroom-instant
    pub async fn create_instant_room(
        &self,
        room_jid: &FullJid,
        show: Option<Show>,
        caps: Option<xmpp_parsers::caps::Caps>,
    ) -> Result<RoomOccupancy, RequestError> {
        // https://xmpp.org/extensions/xep-0045.html#createroom
        let occupancy = self
            .send_presence_to_room(&room_jid, None, show, caps)
            .await?;

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
        show: Option<Show>,
        caps: Option<xmpp_parsers::caps::Caps>,
        handler: impl FnOnce(DataForm) -> T,
    ) -> Result<RoomOccupancy, RequestError>
    where
        T: Future<Output = Result<RoomConfigResponse>> + SendUnlessWasm + 'static,
    {
        // https://xmpp.org/extensions/xep-0045.html#createroom
        let occupancy = self
            .send_presence_to_room(&room_jid, None, show, caps)
            .await?;

        // If the room existed already we don't need to proceed…
        if !occupancy.user.status.contains(&Status::RoomHasBeenCreated) {
            return Ok(occupancy);
        }

        self.configure_room(&room_jid.to_bare(), handler).await?;

        Ok(occupancy)
    }

    /// Subsequent Room Configuration
    /// https://xmpp.org/extensions/xep-0045.html#roomconfig
    pub async fn configure_room<T>(
        &self,
        room_jid: &BareJid,
        handler: impl FnOnce(DataForm) -> T,
    ) -> Result<(), RequestError>
    where
        T: Future<Output = Result<RoomConfigResponse>> + SendUnlessWasm + 'static,
    {
        let iq = Iq::from_get(
            self.ctx.generate_id(),
            muc::Query {
                role: muc::query::Role::Owner,
                payloads: vec![],
            },
        )
        .with_to(room_jid.clone().into());

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
        .with_to(room_jid.clone().into());

        self.ctx.send_iq(iq).await?;

        Ok(())
    }

    /// Destroys a room.
    /// https://xmpp.org/extensions/xep-0045.html#destroyroom
    pub async fn destroy_room(
        &self,
        jid: &BareJid,
        alternate_room: Option<&BareJid>,
    ) -> Result<(), RequestError> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            Query::new(Role::Owner).with_payload(Destroy {
                jid: alternate_room.cloned(),
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
        let message = Message::new().set_to(to).add_payload(direct_invite);
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
        let message = Message::new()
            .set_to(room_jid.clone())
            .add_payload(mediated_invite);
        self.ctx.send_stanza(message)?;
        Ok(())
    }

    pub async fn set_room_subject(&self, room_jid: &BareJid, subject: Option<&str>) -> Result<()> {
        let message = Message::new()
            .set_id(self.ctx.generate_id().into())
            .set_type(MessageType::Groupchat)
            .set_to(room_jid.clone())
            .set_subject(subject.unwrap_or_default()); // Send empty string for empty subject
        self.ctx.send_stanza(message)?;
        Ok(())
    }
}

impl MUC {
    async fn send_presence_to_room(
        &self,
        room_jid: &FullJid,
        password: Option<&str>,
        show: Option<Show>,
        caps: Option<xmpp_parsers::caps::Caps>,
    ) -> Result<RoomOccupancy, RequestError> {
        let mut presence = Presence::new(presence::Type::None)
            .with_to(room_jid.clone())
            .with_payloads(vec![Element::builder("x", ns::MUC)
                .append_all(
                    password.map(|password| Element::builder("password", ns::MUC).append(password)),
                )
                .append(
                    Element::builder("history", ns::MUC)
                        .attr("maxstanzas", "0")
                        .build(),
                )
                .build()]);
        presence.show = show;

        if let Some(caps) = caps {
            presence.add_payload(caps)
        }

        let (mut self_presence, presences, message_history, subject) = self
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
            subject,
            message_history,
        })
    }
}

/// Order of events (https://xmpp.org/extensions/xep-0045.html#order)
///   1. In-room presence from other occupants
///   2. In-room presence from the joining entity itself (so-called "self-presence")
///   3. Room history (if any)
///   4. The room subject
///   5. Live messages, presence updates, new user joins, etc.
struct JoinRoomState {
    pub room_jid: FullJid,
    pub presences: Vec<Presence>,
    pub self_presence: Option<Presence>,
    pub subject: Option<String>,
    pub message_history: Vec<Message>,
}

impl RequestFuture<JoinRoomState, (Presence, Vec<Presence>, Vec<Message>, Option<String>)> {
    pub fn new_join_room_request(room_jid: FullJid) -> Self {
        let room_bare_jid = room_jid.to_bare();

        RequestFuture::new(
            format!("MUC {room_jid}"),
            JoinRoomState {
                room_jid,
                presences: vec![],
                self_presence: None,
                subject: None,
                message_history: vec![],
            },
            move |state, element| {
                match element {
                    XMPPElement::Presence(presence) => {
                        let Some(from) = &presence
                            .from
                            .as_ref()
                            .and_then(|from| from.try_as_full().ok())
                        else {
                            return Ok(ElementReducerPoll::Pending(Some(presence.into())));
                        };

                        // Make sure that the presence is actually sent by our room…
                        if from.to_bare() != room_bare_jid {
                            return Ok(ElementReducerPoll::Pending(Some(presence.into())));
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

                        if is_self_presence {
                            state.self_presence = Some(presence.clone());
                        } else {
                            state.presences.push(presence.clone());
                        }

                        Ok(ElementReducerPoll::Pending(None))
                    }
                    XMPPElement::Message(message) => {
                        // Make sure that the message is actually sent by our room and is
                        // not a MAM message. Otherwise, we might run into a situation where - when
                        // a MAM request is performed at the same time as we're connecting to the
                        // same room - we're consuming all messages from the MAM request.
                        if message.from.as_ref().map(|jid| jid.to_bare()).as_ref()
                            != Some(&room_bare_jid)
                            || message.is_mam_message()
                        {
                            return Ok(ElementReducerPoll::Pending(Some(message.into())));
                        }

                        if let Some(subject) = message.subject() {
                            // We're done…
                            state.subject = (!subject.is_empty()).then_some(subject.to_string());
                            return Ok(ElementReducerPoll::Ready);
                        }

                        state.message_history.push(message.clone());
                        Ok(ElementReducerPoll::Pending(None))
                    }
                    XMPPElement::IQ(_) | XMPPElement::PubSubMessage(_) => {
                        Ok(ElementReducerPoll::Pending(Some(element)))
                    }
                }
            },
            |state| {
                (
                    state
                        .self_presence
                        .expect("Internal error. Missing response in PresenceFutureState."),
                    state.presences,
                    state.message_history,
                    state.subject,
                )
            },
        )
    }
}
