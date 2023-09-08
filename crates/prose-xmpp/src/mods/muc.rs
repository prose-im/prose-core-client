// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::client::ModuleContext;
use crate::event::Event as ClientEvent;
use crate::mods::Module;
use crate::stanza::muc::query::{Destroy, Role};
use crate::stanza::muc::{DirectInvite, Query};
use crate::stanza::{muc, Message};
use crate::util::{ElementReducerPoll, RequestError, RequestFuture, XMPPElement};
use crate::{ns, SendUnlessWasm};
use anyhow::Result;
use jid::{BareJid, FullJid, Jid, NodePart, ResourcePart};
use minidom::Element;
use std::future::Future;
use xmpp_parsers::data_forms::{DataForm, DataFormType};
use xmpp_parsers::disco::{DiscoItemsQuery, DiscoItemsResult};
use xmpp_parsers::iq::Iq;
use xmpp_parsers::muc::user::Status;
use xmpp_parsers::muc::MucUser;
use xmpp_parsers::presence;
use xmpp_parsers::presence::Presence;
use xmpp_parsers::stanza_error::StanzaError;

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

impl Module for MUC {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }

    fn handle_message_stanza(&self, stanza: &Message) -> Result<()> {
        let (Some(from), Some(direct_invite)) = (&stanza.from, &stanza.direct_invite) else {
            return Ok(());
        };

        self.ctx
            .schedule_event(ClientEvent::MUC(Event::DirectInvite {
                from: from.clone(),
                invite: direct_invite.clone(),
            }));

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// XEP-0249: Direct MUC Invitations
    DirectInvite { from: Jid, invite: DirectInvite },
}

#[derive(Debug, Clone, PartialEq)]
pub enum RoomConfigResponse {
    Submit(DataForm),
    Cancel,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Room already exists")]
    RoomAlreadyExists,
    #[error(transparent)]
    RequestError(#[from] RequestError),
    #[error(transparent)]
    ParseError(#[from] xmpp_parsers::Error),
    #[error(transparent)]
    JidError(#[from] jid::Error),
    #[error("Handler returned with error {0}")]
    HandlerError(String),
}

impl MUC {
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

    pub async fn enter_room(&self, room_jid: &BareJid, nickname: impl AsRef<str>) -> Result<()> {
        let full_room_jid = room_jid.with_resource_str(nickname.as_ref())?;
        self.send_presence_to_room(&full_room_jid).await?;
        Ok(())
    }

    pub async fn create_instant_room(
        &self,
        service: &BareJid,
        room_name: impl AsRef<str>,
        nickname: impl AsRef<str>,
    ) -> Result<()> {
        let room_jid = Self::build_room_jid_full(service, room_name, nickname)?;
        self.create_room(&room_jid).await?;

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
        Ok(())
    }

    /// https://xmpp.org/extensions/xep-0045.html#createroom
    pub async fn create_reserved_room<T>(
        &self,
        service: &BareJid,
        room_name: impl AsRef<str>,
        nickname: impl AsRef<str>,
        handler: impl FnOnce(DataForm) -> T,
    ) -> Result<(), Error>
    where
        T: Future<Output = Result<RoomConfigResponse>> + SendUnlessWasm + 'static,
    {
        let room_jid = Self::build_room_jid_full(service, room_name, nickname)?;
        self.create_room(&room_jid).await?;

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

        let handler_result = handler(form)
            .await
            .map_err(|e| Error::HandlerError(e.to_string()))?;

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

        Ok(())
    }

    pub async fn destroy_room(&self, jid: &BareJid) -> Result<()> {
        let iq = Iq::from_set(
            self.ctx.generate_id(),
            Query::new(Role::Owner).with_payload(Destroy {
                jid: jid.clone(),
                reason: None,
            }),
        );
        self.ctx.send_iq(iq).await?;
        Ok(())
    }

    pub fn build_room_jid_full(
        service: &BareJid,
        room_name: impl AsRef<str>,
        nickname: impl AsRef<str>,
    ) -> Result<FullJid, jid::Error> {
        Ok(FullJid::from_parts(
            Some(&NodePart::new(
                &room_name.as_ref().to_ascii_lowercase().replace(" ", "-"),
            )?),
            &service.domain(),
            &ResourcePart::new(&nickname.as_ref().to_ascii_lowercase().replace(" ", "-"))?,
        ))
    }

    pub fn build_room_jid_bare(
        service: &BareJid,
        room_name: impl AsRef<str>,
    ) -> Result<BareJid, jid::Error> {
        Ok(BareJid::from_parts(
            Some(&NodePart::new(
                &room_name.as_ref().to_ascii_lowercase().replace(" ", "-"),
            )?),
            &service.domain(),
        ))
    }
}

impl MUC {
    /// Sends `presence` and returns the next received received presence stanza that matches the
    /// `to` attribute of `presence`.
    async fn send_presence(&self, presence: Presence) -> Result<Presence, RequestError> {
        let Some(Jid::Full(to)) = presence.to.clone() else {
            return Err(RequestError::Generic {
                msg: "Expected FullJid for `to` for sending presence exchange.".to_string(),
            });
        };

        self.ctx
            .send_stanza_with_future(presence, RequestFuture::new_presence_request(to))
            .await
    }

    async fn send_presence_to_room(&self, room_jid: &FullJid) -> Result<MucUser, Error> {
        let presence = Presence::new(presence::Type::None)
            .with_to(room_jid.clone())
            .with_payloads(vec![Element::builder("x", ns::MUC).build()]);

        let mut response = self.send_presence(presence).await?;
        let payload = response
            .payloads
            .pop()
            .ok_or(RequestError::UnexpectedResponse)?;

        Ok(MucUser::try_from(payload)?)
    }

    /// https://xmpp.org/extensions/xep-0045.html#createroom
    async fn create_room(&self, room_jid: &FullJid) -> Result<MucUser, Error> {
        let user = self.send_presence_to_room(room_jid).await?;

        if !user.status.contains(&Status::RoomHasBeenCreated) {
            return Err(Error::RoomAlreadyExists);
        }

        Ok(user)
    }

    async fn send_direct_invite(
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
}

struct PresenceFutureState {
    pub to: FullJid,
    pub response: Option<Presence>,
}

impl RequestFuture<PresenceFutureState, Presence> {
    pub fn new_presence_request(to: FullJid) -> Self {
        RequestFuture::new(
            PresenceFutureState {
                to: to.clone(),
                response: None,
            },
            |state, element| {
                let XMPPElement::Presence(presence) = element else {
                    return Ok(ElementReducerPoll::Pending);
                };

                if presence.from != Some(Jid::Full(state.to.clone())) {
                    return Ok(ElementReducerPoll::Pending);
                }

                if presence.type_ == presence::Type::Error {
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

                state.response = Some(presence.clone());
                Ok(ElementReducerPoll::Ready)
            },
            |state| {
                state
                    .response
                    .expect("Internal error. Missing response in PresenceFutureState.")
            },
        )
    }
}
