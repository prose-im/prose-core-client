// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use libstrophe::{Connection, Error, Stanza};

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
        let result: Result<(), Error> = match stanza_type {
            Some("chat") | None => {
                log::debug!("[message] got chat stanza");

                Self::handle_chat(connection, stanza)
            }
            Some("groupchat") => {
                log::debug!("[message] got groupchat stanza");

                Self::handle_groupchat(connection, stanza)
            }
            Some("normal") => {
                log::debug!("[message] got normal stanza");

                // Alias to 'chat'
                Self::handle_normal(connection, stanza)
            }
            Some("headline") => {
                log::debug!("[message] got headline stanza");

                Self::handle_headline(connection, stanza)
            }
            Some("error") => {
                log::debug!("[message] got error stanza");

                Ok(())
            }
            _ => {
                // Type not handled
                log::warn!("[message] got unhandled type: {:?}", stanza_type);

                Ok(())
            }
        };

        if let Err(err) = result {
            log::error!("[message] handle error: {}", err);
        }
    }

    fn handle_chat(_connection: &mut Connection, _stanza: &Stanza) -> Result<(), Error> {
        // TODO

        Ok(())
    }

    fn handle_groupchat(_connection: &mut Connection, _stanza: &Stanza) -> Result<(), Error> {
        // TODO

        Ok(())
    }

    fn handle_normal(connection: &mut Connection, stanza: &Stanza) -> Result<(), Error> {
        // Alias to 'chat'
        Self::handle_chat(connection, stanza)
    }

    fn handle_headline(_connection: &mut Connection, _stanza: &Stanza) -> Result<(), Error> {
        // TODO

        Ok(())
    }
}
