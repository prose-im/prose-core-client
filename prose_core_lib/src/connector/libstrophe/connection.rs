use std::sync::mpsc::Sender;
use std::sync::Mutex;
use tracing::error;

use crate::connector::Connection;
use crate::helpers::id_string_macro::id_string;

use super::ContextMessage;

id_string!(ConnectionId);

pub(crate) struct LibstropheConnection {
    connection_id: ConnectionId,
    connection_sender: Mutex<Sender<ConnectionMessage>>,
    context_sender: Mutex<Sender<ContextMessage>>,
}

impl LibstropheConnection {
    pub(crate) fn new(
        connection_id: ConnectionId,
        connection_sender: Sender<ConnectionMessage>,
        context_sender: Sender<ContextMessage>,
    ) -> Self {
        LibstropheConnection {
            connection_id,
            connection_sender: Mutex::new(connection_sender),
            context_sender: Mutex::new(context_sender),
        }
    }
}

pub(crate) enum ConnectionMessage {
    SendStanza(libstrophe::Stanza),
}

impl Connection for LibstropheConnection {
    fn disconnect(&self) {
        match self
            .context_sender
            .lock()
            .unwrap()
            .send(ContextMessage::Stop(self.connection_id.clone()))
        {
            Ok(..) => return,
            Err(err) => {
                error!("Could not send disconnect message {}", err.to_string())
            }
        }
    }

    fn send_stanza(&self, stanza: libstrophe::Stanza) {
        self.connection_sender
            .lock()
            .unwrap()
            .send(ConnectionMessage::SendStanza(stanza))
            .unwrap()
    }
}
