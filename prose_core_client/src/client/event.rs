// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use libstrophe::{Connection, ConnectionEvent, Context, Stanza};

use crate::protocol::iq::ProseProtocolIQ;
use crate::protocol::message::ProseProtocolMessage;
use crate::protocol::presence::ProseProtocolPresence;

// -- Structures --

pub struct ProseClientEvent;

// -- Implementations --

impl ProseClientEvent {
    pub fn connection(context: &Context, connection: &mut Connection, event: ConnectionEvent) {
        if let Some(jid) = connection.jid() {
            match event {
                ConnectionEvent::RawConnect => {
                    log::trace!("[event] connected (raw) -> {}", jid);

                    // Nothing done here (as we never use raw connections)
                }
                ConnectionEvent::Connect => {
                    log::trace!("[event] connected -> {}", jid);

                    // TODO: register connection to event broker

                    // Bind stanza handlers
                    connection.handler_add(Self::stanza_presence, None, Some("presence"), None);
                    connection.handler_add(Self::stanza_message, None, Some("message"), None);
                    connection.handler_add(Self::stanza_iq, None, Some("iq"), None);

                    // Announce first presence
                    // TODO: this should not be done from here, right?
                    // TODO: set resource + other metas in presence
                    let presence = Stanza::new_presence();

                    connection.send(&presence);
                }
                ConnectionEvent::Disconnect(err) => {
                    log::trace!("[event] disconnected -> {}: {:?}", jid, err);

                    // TODO: unregister connection from event broker? (or \
                    //   should we just reconnect forever)

                    context.stop();
                }
            }
        }
    }

    pub fn stanza_presence(
        _context: &Context,
        connection: &mut Connection,
        stanza: &Stanza,
    ) -> bool {
        log::trace!("[event] presence from: {}", stanza.from().unwrap_or("--"));

        // Route stanza to presence handler
        ProseProtocolPresence::handle(connection, stanza);

        true
    }

    pub fn stanza_message(
        _context: &Context,
        connection: &mut Connection,
        stanza: &Stanza,
    ) -> bool {
        log::trace!("[event] message from: {}", stanza.from().unwrap_or("--"));

        // Route stanza to message handler
        ProseProtocolMessage::handle(connection, stanza);

        true
    }

    pub fn stanza_iq(_context: &Context, connection: &mut Connection, stanza: &Stanza) -> bool {
        log::trace!("[event] iq from: {}", stanza.from().unwrap_or("--"));

        // Route stanza to IQ handler
        ProseProtocolIQ::handle(connection, stanza);

        true
    }
}
