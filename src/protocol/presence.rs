// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use libstrophe::{Connection, Stanza};

// -- Structures --

pub struct ProseProtocolPresence;

// -- Implementations --

impl ProseProtocolPresence {
    pub fn handle(_connection: &mut Connection, stanza: &Stanza) {
        let stanza_type = stanza.stanza_type();

        // TODO: check if personal, or MUC from here

        // Handle presence type
        // Notice: consider 'available' as empty (because it is illegal as per \
        //   specification, but some XMPP clients are using it)
        // @ref: https://xmpp.org/rfcs/rfc3921.html#stanzas
        match stanza_type {
            Some("available") | None => {
                log::debug!("[presence] got available stanza");
            }
            Some("unavailable") => {
                log::debug!("[presence] got unavailable stanza");
            }
            Some("subscribe") => {
                log::debug!("[presence] got subscribe stanza");
            }
            Some("subscribed") => {
                log::debug!("[presence] got subscribed stanza");
            }
            Some("unsubscribe") => {
                log::debug!("[presence] got unsubscribe stanza");
            }
            Some("unsubscribed") => {
                log::debug!("[presence] got unsubscribed stanza");
            }
            Some("probe") => {
                log::debug!("[presence] got probe stanza");
            }
            Some("error") => {
                log::debug!("[presence] got error stanza");
            }
            _ => {
                // Type not handled
                log::warn!("[presence] got unhandled type: {:?}", stanza_type);
            }
        }
    }
}
