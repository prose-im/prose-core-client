// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use libstrophe::{Connection, ConnectionEvent, ConnectionFlags, Context, Stanza};

use crate::protocol::namespaces::*;

// -- Structures --

pub struct ProseClientEvent;

// -- Implementations --

impl ProseClientEvent {
    pub fn connection(context: &Context, connection: &mut Connection, event: ConnectionEvent) {
        match event {
            ConnectionEvent::RawConnect => {
                log::trace!("[event] connected (raw)");

                // Nothing done here (as we never use raw connections)
            }
            ConnectionEvent::Connect => {
                log::trace!("[event] connected");

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
                log::trace!("[event] disconnected: {:?}", err);

                context.stop();
            }
        }
    }

    pub fn stanza_presence(
        context: &Context,
        connection: &mut Connection,
        stanza: &Stanza,
    ) -> bool {
        log::trace!("[event] presence from: {}", stanza.from().unwrap_or("--"));

        // TODO

        true
    }

    pub fn stanza_message(context: &Context, connection: &mut Connection, stanza: &Stanza) -> bool {
        log::trace!("[event] message from: {}", stanza.from().unwrap_or("--"));

        // TODO

        true
    }

    pub fn stanza_iq(context: &Context, connection: &mut Connection, stanza: &Stanza) -> bool {
        log::trace!("[event] iq from: {}", stanza.from().unwrap_or("--"));

        // Handle XMLNS from IQ stanza
        // TODO: move to an iterative method using 'get_child_by_ns()', where \
        //   a pre-defined array is scanned, and scan stops whenever xmlns \
        //   found, and handler for this xmlns is called.
        match stanza.ns() {
            Some(NS_VERSION) => {
                // TODO: handle NS_VERSION
                // TODO: handle from 'query' sub-element
            }
            Some(NS_LAST) => {
                // TODO: handle NS_LAST
                // TODO: handle from 'query' sub-element
            }
            Some(NS_URN_TIME) => {
                // TODO: handle NS_URN_TIME
                // TODO: handle from 'time' sub-element
            }
            Some(NS_URN_PING) => {
                // TODO: handle NS_URN_PING
                // TODO: handle from 'ping' sub-element
            }
            Some(DISCO_INFO) => {
                // TODO: handle DISCO_INFO
                // TODO: handle from 'query' sub-element
            }
            _ => {
                // TODO: handle unsupported
                // TODO: reply not implemented only if not error IQ
            }
        }

        true
    }
}
