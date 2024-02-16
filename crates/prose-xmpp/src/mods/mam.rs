// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::future::Future;

use anyhow::Result;
use jid::BareJid;
use tracing::error;
use xmpp_parsers::data_forms::{DataForm, DataFormType, Field};
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::rsm::SetQuery;

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::stanza::message::{mam, stanza_id};
use crate::stanza::ns;
use crate::util::{ElementReducerPoll, RequestError, RequestFuture, XMPPElement};

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
    pub fn load_messages_in_chat<'a>(
        &self,
        jid: &BareJid,
        before: impl Into<Option<&'a stanza_id::Id>>,
        after: impl Into<Option<&'a stanza_id::Id>>,
        max_count: impl Into<Option<usize>>,
    ) -> impl Future<Output = Result<(Vec<mam::ArchivedMessage>, mam::Fin), RequestError>> {
        let query_id = mam::QueryId(self.ctx.generate_id());
        let id = self.ctx.generate_id();

        let mut before = before.into().map(ToString::to_string);
        let after = after.into().map(ToString::to_string);

        if before.is_none() && after.is_none() {
            before = Some("".to_string())
        }

        let iq = Iq::from_set(
            id.clone(),
            mam::Query {
                queryid: Some(query_id.clone()),
                node: None,
                form: Some(DataForm::new(
                    DataFormType::Submit,
                    ns::MAM,
                    vec![Field::text_single("with", &jid.to_string())],
                )),
                set: Some(SetQuery {
                    max: max_count.into(),
                    after,
                    before,
                    index: None,
                }),
                flip_page: false,
            },
        );

        self.ctx
            .send_stanza_with_future(iq, RequestFuture::new_mam_request(id, query_id))
    }

    pub fn load_messages_in_muc_chat<'a>(
        &self,
        room_id: &BareJid,
        before: impl Into<Option<&'a stanza_id::Id>>,
        after: impl Into<Option<&'a stanza_id::Id>>,
        max_count: impl Into<Option<usize>>,
    ) -> impl Future<Output = Result<(Vec<mam::ArchivedMessage>, mam::Fin), RequestError>> {
        let query_id = mam::QueryId(self.ctx.generate_id());
        let id = self.ctx.generate_id();

        let mut before = before.into().map(ToString::to_string);
        let after = after.into().map(ToString::to_string);

        if before.is_none() && after.is_none() {
            before = Some("".to_string())
        }

        let iq = Iq::from_set(
            id.clone(),
            mam::Query {
                queryid: Some(query_id.clone()),
                node: None,
                form: None,
                set: Some(SetQuery {
                    max: max_count.into(),
                    after,
                    before,
                    index: None,
                }),
                flip_page: false,
            },
        )
        .with_to(room_id.clone().into());

        self.ctx
            .send_stanza_with_future(iq, RequestFuture::new_mam_request(id, query_id))
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
