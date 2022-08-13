// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::connection::xmpp_connection::{ConnectionEvent, ConnectionHandler, XMPPSender};
use crate::connection::{StanzaHandler, XMPPConnection};
use crate::error::Result;
use jid::FullJid;
use libstrophe::{Connection, ConnectionFlags, Context, Logger, Stanza};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub(crate) struct LibstropheConnection {
    connection: Connection<'static, 'static>,
    connection_handler: Option<ConnectionHandler>,
    stanza_handler: Option<StanzaHandler>,
}

impl LibstropheConnection {
    pub(crate) fn new(jid: &FullJid, password: &str) -> LibstropheConnection {
        let logger = Logger::default();

        let mut connection = Connection::new(Context::new(logger));
        connection.set_jid(jid.to_string());
        connection.set_pass(password);

        LibstropheConnection {
            connection,
            connection_handler: None,
            stanza_handler: None,
        }
    }
}

impl<'a, 'b> XMPPConnection for LibstropheConnection {
    fn connect(mut self: Box<Self>) -> Result<Box<dyn XMPPSender>> {
        let (tx, rx) = channel::<Stanza>();

        let mut connection_handler = self.connection_handler.unwrap_or(Box::new(|_| ()));

        let connection_handler =
            move |_: &Context, _: &mut Connection, event: libstrophe::ConnectionEvent| {
                let connection_event = match event {
                    libstrophe::ConnectionEvent::RawConnect => ConnectionEvent::Connect,
                    libstrophe::ConnectionEvent::Connect => ConnectionEvent::Connect,
                    libstrophe::ConnectionEvent::Disconnect(err) => {
                        ConnectionEvent::Disconnect(err.map(Into::into))
                    }
                };
                connection_handler(&connection_event);
            };

        let send_handler = move |_: &Context, conn: &mut Connection| {
            match rx.try_recv() {
                Ok(stanza) => conn.send(&stanza),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => return false,
            }
            true
        };

        self.connection.set_flags(ConnectionFlags::TRUST_TLS)?;
        self.connection
            .timed_handler_add(send_handler, Duration::from_millis(1));

        if let Some(mut stanza_handler) = self.stanza_handler {
            self.connection.handler_add(
                move |_, _, stanza| {
                    stanza_handler(stanza);
                    true
                },
                None,
                None,
                None,
            );
        }

        let ctx = self
            .connection
            .connect_client(None, None, connection_handler)?;
        let thread = thread::Builder::new()
            .name("org.prose.xmpp-thread".to_string())
            .spawn(move || ctx.run())?;

        Ok(Box::new(LibstropheContext {
            _thread: thread,
            message_channel: tx,
        }))
    }

    fn set_connection_handler(&mut self, handler: ConnectionHandler) {
        self.connection_handler = Some(handler);
    }

    fn set_stanza_handler(&mut self, handler: StanzaHandler) {
        self.stanza_handler = Some(handler);
    }
}

pub(crate) struct LibstropheContext {
    _thread: JoinHandle<()>,
    message_channel: Sender<Stanza>,
}

impl XMPPSender for LibstropheContext {
    fn send_stanza(&self, stanza: Stanza) -> Result<()> {
        self.message_channel.send(stanza)?;
        Ok(())
    }
}
