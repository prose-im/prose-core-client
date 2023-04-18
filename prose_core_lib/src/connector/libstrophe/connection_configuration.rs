use std::arch::asm;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

use tracing::info;

use crate::connector::ConnectionConfiguration;
use crate::{ConnectionError, ConnectionEvent};

use super::{ConnectionId, ConnectionMessage, ContextMessage, LibstropheConnection};

impl ConnectionConfiguration {
    pub(crate) fn configure_and_connect<'cb, 'cx>(
        self,
        connection_id: ConnectionId,
        connection: libstrophe::Connection<'cb, 'cx>,
        connection_sender: Sender<ConnectionMessage>,
        context_sender: Sender<ContextMessage>,
        connection_receiver: Receiver<ConnectionMessage>,
    ) -> anyhow::Result<libstrophe::Context<'cx, 'cb>> {
        let mut connection = connection;
        connection.set_flags(libstrophe::ConnectionFlags::TRUST_TLS)?;
        connection.set_jid(self.jid.to_string());
        connection.set_pass(self.password);

        connection
            .timed_handler_add(
                move |_, conn| {
                    match connection_receiver.try_recv() {
                        Ok(ConnectionMessage::SendStanza(stanza)) => conn.send(&stanza),
                        Err(TryRecvError::Empty) => {}
                        Err(TryRecvError::Disconnected) => return false,
                    }
                    true
                },
                Duration::from_millis(1),
            )
            .expect("Could not install send handler");

        {
            let conn = LibstropheConnection::new(
                connection_id.clone(),
                connection_sender.clone(),
                context_sender.clone(),
            );
            let mut timeout_handler = self.timeout_handler;

            connection
                .timed_handler_add(
                    move |_, _| {
                        // This is a stupid hack to prevent the compiler from optimizing our
                        // closure. Otherwise it leads sometimes to the libstrophe wrapper not
                        // being able to disambiguate it from our other closures.
                        unsafe {
                            asm!("nop");
                        }
                        timeout_handler(&conn)
                    },
                    Duration::from_secs(2),
                )
                .expect("Could not install timeout handler");
        }

        {
            let conn = LibstropheConnection::new(
                connection_id.clone(),
                connection_sender.clone(),
                context_sender.clone(),
            );
            let mut ping_handler = self.ping_handler;

            connection
                .timed_handler_add(move |_, _| ping_handler(&conn), Duration::from_secs(60))
                .expect("Could not install ping handler");
        }

        {
            let conn = LibstropheConnection::new(
                connection_id.clone(),
                connection_sender.clone(),
                context_sender.clone(),
            );
            let mut stanza_handler = self.stanza_handler;

            connection
                .handler_add(
                    move |_, _, stanza| {
                        stanza_handler(&conn, stanza);
                        true
                    },
                    None,
                    None,
                    None,
                )
                .expect("Could not install stanza handler");
        }

        let conn = LibstropheConnection::new(
            connection_id.clone(),
            connection_sender.clone(),
            context_sender.clone(),
        );
        let mut connection_handler = self.connection_handler;
        let mut connection_established = false;

        let now = Instant::now();
        info!("Connecting to server via libstrophe…");

        let ctx = connection
            .connect_client(None, None, move |_, _, event| {
                let connection_event = match event {
                    libstrophe::ConnectionEvent::RawConnect => return,
                    libstrophe::ConnectionEvent::Connect => {
                        connection_established = true;
                        ConnectionEvent::Connect
                    }
                    libstrophe::ConnectionEvent::Disconnect(None) => ConnectionEvent::Disconnect {
                        error: if connection_established {
                            ConnectionError::TimedOut
                        } else {
                            ConnectionError::InvalidCredentials {}
                        },
                    },
                    libstrophe::ConnectionEvent::Disconnect(Some(err)) => {
                        ConnectionEvent::Disconnect { error: err.into() }
                    }
                };
                connection_handler(&conn, &connection_event);
            })
            .map_err(|err| anyhow::Error::new(err.error))?;

        info!(
            "Established libstrophe connection after {:.2?}",
            now.elapsed()
        );

        Ok(ctx)
    }
}
