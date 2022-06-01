// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use libstrophe::{Connection, Error, Stanza, StanzaRef};

use super::{builders::ProseProtocolBuilders, namespaces};

// -- Structures --

pub struct ProseProtocolIQ;

// -- Implementations --

impl ProseProtocolIQ {
    pub fn handle(connection: &mut Connection, stanza: &Stanza) {
        let stanza_type = stanza.stanza_type();

        // Handle request type
        // Notice: consider empty types as 'get'
        match stanza_type {
            Some("get") | None => {
                for child in stanza.children() {
                    // A child must have a name, drop the invalid ones
                    if let Some(name) = child.name() {
                        let result = Self::handle_get(connection, name, stanza, &child);

                        if let Err(err) = result {
                            log::error!("[iq] handle get error: {}", err);
                        }
                    }
                }
            }
            _ => {
                // Type not handled
                log::warn!("[iq] got unhandled type: {:?}", stanza_type);
            }
        }
    }

    fn handle_get(
        connection: &mut Connection,
        name: &str,
        stanza: &Stanza,
        child: &StanzaRef,
    ) -> Result<(), Error> {
        // Handle XMLNS from 'get' type
        match (name, child.ns()) {
            ("query", Some(namespaces::NS_VERSION)) => {
                log::debug!("[iq] got version request");

                Self::handle_get_version(connection, stanza)
            }
            ("query", Some(namespaces::NS_LAST)) => {
                log::debug!("[iq] got last activity request");

                Self::handle_get_last(connection, stanza)
            }
            ("query", Some(namespaces::DISCO_INFO)) => {
                log::debug!("[iq] got discovery information request");

                Self::handle_get_disco_info(connection, stanza)
            }
            ("time", Some(namespaces::NS_URN_TIME)) => {
                log::debug!("[iq] got local time request");

                Self::handle_get_time(connection, stanza)
            }
            ("ping", Some(namespaces::NS_URN_PING)) => {
                log::debug!("[iq] got ping request");

                Self::handle_get_ping(connection, stanza)
            }
            _ => {
                log::warn!("[iq] got unsupported request");

                // TODO: handle unsupported
                // TODO: reply not implemented only if not error IQ

                // TODO
                Ok(())
            }
        }
    }

    fn handle_get_version(connection: &mut Connection, stanza: &Stanza) -> Result<(), Error> {
        // @ref: https://xmpp.org/extensions/xep-0092.html

        // TODO: build a grand-macro to generate stanzas

        // Reply with version
        // TODO: populate w/ final values
        connection.send(&ProseProtocolBuilders::stanza_reply(
            stanza,
            vec![ProseProtocolBuilders::stanza_named_ns(
                "query",
                namespaces::NS_VERSION,
                vec![
                    ProseProtocolBuilders::stanza_named(
                        "name",
                        vec![ProseProtocolBuilders::stanza_text("Prose")?],
                    )?,
                    ProseProtocolBuilders::stanza_named(
                        "version",
                        vec![ProseProtocolBuilders::stanza_text("0.0.0")?],
                    )?,
                    ProseProtocolBuilders::stanza_named(
                        "os",
                        vec![ProseProtocolBuilders::stanza_text("macOS 0.0")?],
                    )?,
                ],
            )?],
        )?);

        Ok(())
    }

    fn handle_get_last(_connection: &mut Connection, _stanza: &Stanza) -> Result<(), Error> {
        // @ref: https://xmpp.org/extensions/xep-0012.html

        // TODO

        Ok(())
    }

    fn handle_get_disco_info(_connection: &mut Connection, _stanza: &Stanza) -> Result<(), Error> {
        // @ref: https://xmpp.org/extensions/xep-0030.html

        // TODO

        Ok(())
    }

    fn handle_get_time(_connection: &mut Connection, _stanza: &Stanza) -> Result<(), Error> {
        // @ref: https://xmpp.org/extensions/xep-0202.html

        // TODO

        Ok(())
    }

    fn handle_get_ping(_connection: &mut Connection, _stanza: &Stanza) -> Result<(), Error> {
        // @ref: https://xmpp.org/extensions/xep-0199.html

        // TODO

        Ok(())
    }
}
