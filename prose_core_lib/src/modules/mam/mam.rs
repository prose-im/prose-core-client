use std::collections::HashMap;
use std::sync::Mutex;

use jid::BareJid;

use crate::modules::mam::types::Fin;
use crate::modules::{Module, RequestError};
use crate::stanza::form::Kind::Submit;
use crate::stanza::iq::Kind;
use crate::stanza::{message, Form, ForwardedMessage, Message, Namespace, Stanza, StanzaBase, IQ};

use super::super::Context;

pub struct ArchivedMessage<'a> {
    pub stanza_id: Option<message::StanzaId>,
    pub message: ForwardedMessage<'a>,
}

impl<'a> ArchivedMessage<'a> {
    pub fn clone<'b>(&self) -> ArchivedMessage<'b> {
        ArchivedMessage {
            stanza_id: self.stanza_id.clone(),
            message: self.message.clone(),
        }
    }
}

pub struct MAM {
    // Contains messages received between a sent MAM query and a received <fin> node.
    collected_messages: Mutex<HashMap<String, Vec<ArchivedMessage<'static>>>>,
}

impl MAM {
    pub fn new() -> Self {
        MAM {
            collected_messages: Mutex::new(HashMap::new()),
        }
    }
}

impl Module for MAM {
    fn handle_message_stanza(&self, _ctx: &Context, stanza: &Message) -> anyhow::Result<()> {
        let Some(result_node) =
            stanza.child_by_name_and_namespace("result", Namespace::MAM2) else {
            return Ok(())
        };

        let Some(query_id) = result_node.attribute("queryid") else {
            return Ok(())
        };

        let Some(message) =
            result_node.child_by_name_and_namespace("forwarded", Namespace::Forward)
        else {
            return Ok(())
        };

        self.collected_messages
            .lock()
            .unwrap()
            .entry(query_id.to_string())
            .or_insert(vec![])
            .push(ArchivedMessage {
                stanza_id: result_node.attribute("id").map(Into::into),
                message: message.clone().into(),
            });
        Ok(())
    }
}

impl MAM {
    pub async fn load_messages_in_chat(
        &self,
        ctx: &Context<'_>,
        jid: &BareJid,
        before: impl Into<Option<&message::StanzaId>>,
        after: impl Into<Option<&message::StanzaId>>,
        max_count: impl Into<Option<u32>>,
    ) -> anyhow::Result<(Vec<ArchivedMessage>, Fin)> {
        let query_id = ctx.generate_id();

        let form = Form::new(Submit)
            .set_form_type(Namespace::MAM2.to_string())
            .add_field_with_value("with", jid.to_string(), None);

        let before = before.into();
        let after = after.into();

        let mut result_set = Stanza::new("set").set_namespace(Namespace::RSM);

        if let Some(max_count) = max_count.into() {
            result_set = result_set.add_child(Stanza::new_text_node("max", max_count.to_string()));
        }

        if before.is_none() && after.is_none() {
            // If no message id is set, we'll load the last page.
            // See: https://xmpp.org/extensions/xep-0313.html#sect-idm46520034792112
            result_set = result_set.add_child(Stanza::new("before"))
        }

        if let Some(before) = before {
            result_set = result_set.add_child(Stanza::new_text_node("before", before.to_string()))
        }
        if let Some(after) = after {
            result_set = result_set.add_child(Stanza::new_text_node("after", after.to_string()))
        }

        let query = Stanza::new_query(Namespace::MAM2, Some(&query_id))
            .add_child(form)
            .add_child(result_set);

        self.collected_messages
            .lock()
            .unwrap()
            .insert(query_id.clone(), vec![]);

        let iq = ctx
            .send_iq(IQ::new(Kind::Set, ctx.generate_id()).add_child(query))
            .await?;

        let Some(fin_node) = iq.child_by_name_and_namespace("fin", Namespace::MAM2) else {
            return Err(anyhow::Error::new(RequestError::UnexpectedResponse))
        };

        let messages = self
            .collected_messages
            .lock()
            .unwrap()
            .remove(&query_id)
            .unwrap_or(vec![]);

        Ok((messages, fin_node.clone().into()))
    }
}
