// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::namespace::Namespace;
use crate::AccountObserver;
use crate::Message;
use crate::Presence;
use crate::Roster;
use jid::BareJid;
use libstrophe::{Connection, ConnectionEvent, ConnectionFlags, Context, Stanza};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use strum_macros::Display;

type ThrowingStanzaHandler =
    Box<dyn FnMut(&Context, &mut Connection, &Stanza) -> Result<bool, ()> + Send>;
type StanzaHandler = Box<dyn FnMut(&Context, &mut Connection, &Stanza) -> bool + Send>;

fn to_stanza_handler(mut handler: ThrowingStanzaHandler) -> StanzaHandler {
    Box::new(move |ctx, conn, stanza| match handler(ctx, conn, stanza) {
        Ok(_) => return true,
        Err(_) => return true,
    })
}

#[derive(Debug, thiserror::Error, Display)]
pub enum ConnectionError {
    Err { description: String },
}

pub struct Account {
    message_channel: Sender<Stanza>,
    _thread: JoinHandle<()>,
}

impl Account {
    pub fn new(
        jid: &BareJid,
        password: &str,
        observer: Arc<Box<dyn AccountObserver>>,
    ) -> Result<Account, ConnectionError> {
        let (tx, rx) = channel::<Stanza>();

        let conn_observer = observer.clone();

        let conn_handler =
            move |ctx: &Context, conn: &mut Connection, evt: ConnectionEvent| match evt {
                ConnectionEvent::Connect => {
                    conn_observer.didConnect();

                    // After establishing a session, a client SHOULD send initial presence to the server
                    // in order to signal its availability for communications. As defined herein, the initial
                    // presence stanza (1) MUST possess no 'to' address (signalling that it is meant to be
                    // broadcasted by the server on behalf of the client) and (2) MUST possess no 'type' attribute
                    // (signalling the user's availability). After sending initial presence, an active resource is
                    // said to be an "available resource".
                    let pres = Stanza::new_presence();
                    conn.send(&pres);
                }
                ConnectionEvent::Disconnect(err) => {
                    println!("Disconnected, Reason: {:?}", err);
                    ctx.stop();
                    conn_observer.didDisconnect();
                }
                _ => unimplemented!(),
            };

        let send_handler = move |_ctx: &Context, conn: &mut Connection| {
            match rx.try_recv() {
                Ok(stanza) => conn.send(&stanza),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => return false,
            }
            true
        };

        let message_observer = observer.clone();

        let mut message_handler: ThrowingStanzaHandler = Box::new(move |_, _, stanza| {
            let message: Message = stanza.try_into()?;
            message_observer.as_ref().didReceiveMessage(message);
            Ok(true)
        });

        let result_observer = observer.clone();

        let mut result_handler: ThrowingStanzaHandler = Box::new(move |_, _, stanza| {
            let query = stanza.get_child_by_name("query").ok_or(())?;
            let ns = query.ns().ok_or(())?;

            match ns {
                Namespace::Roster => {
                    let roster: Roster = stanza.try_into()?;
                    result_observer.didReceiveRoster(roster);
                }
                _ => (),
            }
            Ok(true)
        });

        let presence_observer = observer.clone();

        let mut presence_handler: ThrowingStanzaHandler = Box::new(move |_, _, stanza| {
            let presence: Presence = stanza.try_into()?;
            presence_observer.as_ref().didReceivePresence(presence);
            Ok(true)
        });

        let stanza_handler: ThrowingStanzaHandler = Box::new(move |ctx, conn, stanza| {
            let name = stanza.name().ok_or(())?;

            match name {
                "message" => message_handler(ctx, conn, stanza),
                "iq" => result_handler(ctx, conn, stanza),
                "presence" => presence_handler(ctx, conn, stanza),
                _ => Ok(true),
            }
        });

        let mut conn = Connection::new(Context::new_with_default_logger());

        conn.set_flags(ConnectionFlags::TRUST_TLS).unwrap();

        conn.handler_add(to_stanza_handler(stanza_handler), None, None, None)
            .unwrap();

        conn.timed_handler_add(send_handler, Duration::from_millis(1))
            .unwrap();
        conn.set_jid(jid.to_string());
        conn.set_pass(password);
        conn.set_flags(ConnectionFlags::TRUST_TLS).unwrap();

        let ctx =
            conn.connect_client(None, None, conn_handler)
                .map_err(|e| ConnectionError::Err {
                    description: e.error.to_string(),
                })?;
        let thread = thread::Builder::new()
            .spawn(move || ctx.run())
            .map_err(|e| ConnectionError::Err {
                description: e.to_string(),
            })?;

        Ok(Account {
            message_channel: tx,
            _thread: thread,
        })
    }

    pub fn send_message(&self, jid: &BareJid, body: &str) {
        let mut stanza = Stanza::new_message(Some("chat"), None, Some(&jid.to_string()));
        stanza.set_body(&body.to_string()).unwrap();
        self.message_channel.send(stanza).unwrap();
    }

    pub fn load_roster(&self) {
        let mut iq_stanza = Stanza::new_iq(Some("get"), Some("roster1"));
        let mut query = Stanza::new();
        query.set_name("query").unwrap();
        query.set_ns("jabber:iq:roster").unwrap();
        iq_stanza.add_child(query).unwrap();
        self.message_channel.send(iq_stanza).unwrap();
    }

    pub fn send_xml_payload(&self, xml_str: &str) {
        let stanza = Stanza::from_str(xml_str);
        self.message_channel.send(stanza).unwrap();
    }
}
