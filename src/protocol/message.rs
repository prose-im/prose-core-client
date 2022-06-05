// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use libstrophe::{Connection, Stanza};

// -- Structures --

pub struct ProseProtocolMessage;

// -- Implementations --

impl ProseProtocolMessage {
    pub fn handle(connection: &mut Connection, stanza: &Stanza) {
        let stanza_type = stanza.stanza_type();

        // TODO: check if personal, or MUC from here (for DMs)

        // Handle message type
        // Notice: consider empty types as 'chat'
        // @ref: https://xmpp.org/rfcs/rfc3921.html#stanzas
        match stanza_type {
            Some("chat") | None => {
                log::debug!("[message] got chat stanza");
            }
            Some("groupchat") => {
                log::debug!("[message] got groupchat stanza");
            }
            Some("normal") => {
                log::debug!("[message] got normal stanza");
            }
            Some("headline") => {
                log::debug!("[message] got headline stanza");
            }
            Some("error") => {
                log::debug!("[message] got error stanza");
            }
            _ => {
                // Type not handled
                log::warn!("[message] got unhandled type: {:?}", stanza_type);
            }
        }
    }
}
