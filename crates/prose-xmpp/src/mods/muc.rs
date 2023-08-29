// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::ns;
use crate::util::{ElementReducerPoll, RequestError, RequestFuture, XMPPElement};
use anyhow::Result;
use jid::{BareJid, FullJid, Jid};
use minidom::Element;
use xmpp_parsers::disco::{DiscoItemsQuery, DiscoItemsResult};
use xmpp_parsers::iq::Iq;
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

    pub async fn create_instant_room(
        &self,
        service: &BareJid,
        room_name: impl AsRef<str>,
    ) -> Result<()> {
        let room_jid = service.with_resource_str(room_name.as_ref())?;
        let presence = Presence::new(presence::Type::None)
            .with_to(room_jid)
            .with_payloads(vec![Element::builder("x", ns::MUC).build()]);
        let response = self.send_presence(presence).await?;

        println!("{}", String::from(&Element::from(response)));

        // <presence
        //     from='crone1@shakespeare.lit/desktop'
        //     to='coven@chat.shakespeare.lit/firstwitch'>
        //   <x xmlns='http://jabber.org/protocol/muc'/>
        // </presence>

        Ok(())
    }
}

impl MUC {
    /// Sends `presence` and returns the next received received presence stanza that matches the
    /// `to` attribute of `presence`.
    async fn send_presence(&self, presence: Presence) -> Result<Presence, RequestError> {
        let Some(Jid::Full(to)) = presence.to.clone() else {
            return Err(RequestError::Generic { msg: "Expected FullJid for `to` for sending presence exchange.".to_string() })
        };

        self.ctx
            .send_stanza_with_future(presence, RequestFuture::new_presence_request(to))
            .await
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
