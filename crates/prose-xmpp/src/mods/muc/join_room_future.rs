// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::FullJid;
use xmpp_parsers::presence;
use xmpp_parsers::presence::Presence;
use xmpp_parsers::stanza_error::StanzaError;

use crate::stanza::Message;
use crate::util::{ElementReducerPoll, RequestFuture, XMPPElement};
use crate::RequestError;

/// Order of events (https://xmpp.org/extensions/xep-0045.html#order)
///   1. In-room presence from other occupants
///   2. In-room presence from the joining entity itself (so-called "self-presence")
///   3. Room history (if any)
///   4. The room subject
///   5. Live messages, presence updates, new user joins, etc.
pub struct JoinRoomState {
    room_jid: FullJid,
    presences: Vec<Presence>,
    self_presence: Option<Presence>,
    subject: Option<String>,
    message_history: Vec<Message>,
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
                    state.self_presence.unwrap_or_else(|| {
                        panic!(
                            "Internal error. Missing response in JoinRoomState for room {}.",
                            state.room_jid
                        )
                    }),
                    state.presences,
                    state.message_history,
                    state.subject,
                )
            },
        )
    }
}
