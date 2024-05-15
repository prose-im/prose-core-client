// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;

use anyhow::Result;
use jid::BareJid;
use tracing::error;
use xmpp_parsers::iq::{Iq, IqType};

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::stanza::mam::query;
use crate::stanza::message::mam;
use crate::util::{ElementReducerPoll, RequestError, RequestFuture, XMPPElement};

// https://xmpp.org/extensions/xep-0313.html

#[derive(Default, Clone)]
pub struct MAM {
    ctx: ModuleContext,
}

impl Module for MAM {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }
}

impl MAM {
    /// When you're loading message from a MUC chat, make sure to set `to` to the room's JID.
    /// If you're loading messages in a regular conversation, make sure to set the `with` filter
    /// on `query`. Leave `to` blank in this case.
    pub fn load_messages(
        &self,
        to: Option<&BareJid>,
        query: query::Query,
    ) -> impl Future<Output = Result<(Vec<mam::ArchivedMessage>, mam::Fin), RequestError>> {
        let query_id = self.ctx.generate_id();
        let id = self.ctx.generate_id();

        let iq = Iq {
            from: None,
            to: to.map(|jid| jid.clone().into()),
            id: id.clone(),
            payload: IqType::Set(query.into_mam_query(query_id.clone()).into()),
        };

        self.ctx.send_stanza_with_future(
            iq,
            RequestFuture::new_mam_request(id, mam::QueryId(query_id)),
        )
    }
}

struct MAMFutureState {
    id: String,
    query_id: mam::QueryId,
    fin: Option<mam::Fin>,
    messages: Vec<mam::ArchivedMessage>,
}

impl RequestFuture<MAMFutureState, (Vec<mam::ArchivedMessage>, mam::Fin)> {
    fn new_mam_request(id: String, query_id: mam::QueryId) -> Self {
        RequestFuture::new(
            format!("MAM {id}"),
            MAMFutureState {
                id,
                query_id,
                fin: None,
                messages: vec![],
            },
            |state, element| match element {
                XMPPElement::IQ(iq) => {
                    if iq.id != state.id {
                        return Ok(ElementReducerPoll::Pending(Some(iq.into())));
                    }

                    if let IqType::Error(error) = iq.payload {
                        return Err(error.into());
                    }

                    let IqType::Result(Some(payload)) = iq.payload else {
                        return Ok(ElementReducerPoll::Pending(Some(iq.into())));
                    };

                    let fin = match mam::Fin::try_from(payload) {
                        Ok(fin) => fin,
                        Err(err) => {
                            error!("Failed to parse MAM fin element. {}", err.to_string());
                            return Err(RequestError::UnexpectedResponse);
                        }
                    };

                    state.fin = Some(fin);
                    return Ok(ElementReducerPoll::Ready);
                }
                XMPPElement::Message(message) => {
                    let Some(archived_message) = message.archived_message() else {
                        return Ok(ElementReducerPoll::Pending(Some(message.into())));
                    };

                    let Some(query_id) = &archived_message.query_id else {
                        return Ok(ElementReducerPoll::Pending(Some(message.into())));
                    };

                    if query_id != &state.query_id {
                        return Ok(ElementReducerPoll::Pending(Some(message.into())));
                    }

                    state.messages.push(archived_message);
                    Ok(ElementReducerPoll::Pending(None))
                }
                XMPPElement::Presence(_) | XMPPElement::PubSubMessage(_) => {
                    return Ok(ElementReducerPoll::Pending(Some(element)));
                }
            },
            |state| {
                (
                    state.messages,
                    state
                        .fin
                        .expect("Internal error. Missing fin in MAMFutureState."),
                )
            },
        )
    }
}
