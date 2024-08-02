// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

use crate::stanza::Message;
use crate::util::{ElementReducerPoll, RequestFuture, XMPPElement};

pub struct SendMucMessageState {
    room_jid: BareJid,
    sent_message_id: String,
    received_message: Option<Message>,
}

impl RequestFuture<SendMucMessageState, Message> {
    pub fn new_send_muc_message(room_jid: BareJid, sent_message_id: String) -> Self {
        RequestFuture::new(
            format!("MUC Message {sent_message_id} -> {room_jid}"),
            SendMucMessageState {
                room_jid,
                sent_message_id,
                received_message: None,
            },
            move |state, element| match element {
                XMPPElement::Presence(_) | XMPPElement::IQ(_) | XMPPElement::PubSubMessage(_) => {
                    Ok(ElementReducerPoll::Pending(Some(element)))
                }
                XMPPElement::Message(message) => {
                    if message.id.as_ref() != Some(&state.sent_message_id)
                        || message.from.as_ref().map(|from| from.to_bare()).as_ref()
                            != Some(&state.room_jid)
                    {
                        return Ok(ElementReducerPoll::Pending(Some(message.into())));
                    }

                    state.received_message = Some(message);
                    return Ok(ElementReducerPoll::Ready);
                }
            },
            |state| {
                state
                    .received_message
                    .expect("Internal error. Missing received message in SendMucMessageState.")
            },
        )
    }
}
