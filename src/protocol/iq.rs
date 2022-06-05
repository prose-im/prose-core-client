// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use libstrophe::{Connection, Error, Stanza, StanzaRef};

use super::{namespaces, registries, stanza::ProseProtocolStanza};
use crate::utils::macros::map;

// -- Structures --

pub struct ProseProtocolIQ;

// -- Implementations --

impl ProseProtocolIQ {
    pub fn handle(connection: &mut Connection, stanza: &Stanza) {
        let stanza_type = stanza.stanza_type();

        // Handle request type
        // Notice: consider empty types as 'get'
        // @ref: \
        //   https://xmpp.org/extensions/xep-0099.html#sect-idm45805812737232
        match stanza_type {
            Some("get") | None => {
                log::debug!("[iq] got get stanza");

                let (mut count_unsupported, mut total_children) = (0, 0);

                for child in stanza.children() {
                    total_children += 1;

                    // A child must have a name, drop the invalid ones
                    if let Some(name) = child.name() {
                        match Self::handle_get(connection, name, stanza, &child) {
                            Ok(is_unsupported) => {
                                if is_unsupported == true {
                                    count_unsupported += 1;
                                }
                            }
                            Err(err) => {
                                log::error!("[iq] handle get error: {}", err);
                            }
                        }
                    } else {
                        // Consider invalid children as unsupported
                        count_unsupported += 1;
                    }
                }

                // Send unsupported response?
                if count_unsupported == total_children {
                    if let Err(err) = Self::handle_get_unsupported(connection, stanza) {
                        log::error!("[iq] handle get unsupported error: {}", err);
                    }
                }
            }
            Some("set") => {
                log::debug!("[iq] got set stanza");
            }
            Some("result") => {
                log::debug!("[iq] got result stanza");
            }
            Some("error") => {
                log::debug!("[iq] got error stanza");
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
    ) -> Result<bool, Error> {
        let mut is_unsupported = false;

        // Handle XMLNS from 'get' type
        let dispatch_result = match (name, child.ns()) {
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
            ("query", Some(namespaces::DISCO_ITEMS)) => {
                log::debug!("[iq] got discovery items request");

                Self::handle_get_disco_items(connection, stanza)
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

                // Mark request as unsupported
                is_unsupported = true;

                Ok(())
            }
        };

        dispatch_result.and(Ok(is_unsupported))
    }

    fn handle_get_unsupported(connection: &mut Connection, stanza: &Stanza) -> Result<(), Error> {
        // Reply with unsupported error
        connection.send(&ProseProtocolStanza::error(
            stanza,
            "cancel",
            "501",
            "feature-not-implemented",
            "The feature requested is not implemented by the recipient or server and therefore cannot be processed."
        )?);

        Ok(())
    }

    fn handle_get_version(connection: &mut Connection, stanza: &Stanza) -> Result<(), Error> {
        // @ref: https://xmpp.org/extensions/xep-0092.html

        // TODO: build a grand-macro to generate stanzas

        // Reply with version
        // TODO: populate w/ final values
        connection.send(&ProseProtocolStanza::result(
            stanza,
            Some(vec![ProseProtocolStanza::named(
                "query",
                Some(map! { "xmlns" => namespaces::NS_VERSION }),
                Some(vec![
                    ProseProtocolStanza::named(
                        "name",
                        None,
                        Some(vec![ProseProtocolStanza::text("Prose")?]),
                    )?,
                    ProseProtocolStanza::named(
                        "version",
                        None,
                        Some(vec![ProseProtocolStanza::text("0.0.0")?]),
                    )?,
                    ProseProtocolStanza::named(
                        "os",
                        None,
                        Some(vec![ProseProtocolStanza::text("macOS 0.0")?]),
                    )?,
                ]),
            )?]),
        )?);

        Ok(())
    }

    fn handle_get_last(connection: &mut Connection, stanza: &Stanza) -> Result<(), Error> {
        // @ref: https://xmpp.org/extensions/xep-0012.html

        // Reply with version
        // TODO: populate w/ final values
        connection.send(&ProseProtocolStanza::result(
            stanza,
            Some(vec![ProseProtocolStanza::named(
                "query",
                Some(map! { "xmlns" => namespaces::NS_LAST, "seconds" => "42" }),
                None,
            )?]),
        )?);

        Ok(())
    }

    fn handle_get_disco_info(connection: &mut Connection, stanza: &Stanza) -> Result<(), Error> {
        // @ref: https://xmpp.org/extensions/xep-0030.html

        // Generate information children
        let mut children = vec![ProseProtocolStanza::named(
            "identity",
            Some(map! { "category" => "client", "type" => "pc", "name" => "Prose" }),
            None,
        )?];

        for feature in registries::FEATURES {
            children.push(ProseProtocolStanza::named(
                "feature",
                Some(map! { "var" => *feature }),
                None,
            )?);
        }

        // Reply with discovery information
        // TODO: populate w/ final values
        connection.send(&ProseProtocolStanza::result(
            stanza,
            Some(vec![ProseProtocolStanza::named(
                "query",
                Some(map! { "xmlns" => namespaces::DISCO_INFO }),
                Some(children),
            )?]),
        )?);

        Ok(())
    }

    fn handle_get_disco_items(connection: &mut Connection, stanza: &Stanza) -> Result<(), Error> {
        // @ref: https://xmpp.org/extensions/xep-0030.html

        // Reply with discovery items (empty for a client)
        connection.send(&ProseProtocolStanza::result(
            stanza,
            Some(vec![ProseProtocolStanza::named(
                "query",
                Some(map! { "xmlns" => namespaces::DISCO_ITEMS }),
                None,
            )?]),
        )?);

        Ok(())
    }

    fn handle_get_time(connection: &mut Connection, stanza: &Stanza) -> Result<(), Error> {
        // @ref: https://xmpp.org/extensions/xep-0202.html

        // Reply with version
        // TODO: populate w/ final values
        connection.send(&ProseProtocolStanza::result(
            stanza,
            Some(vec![ProseProtocolStanza::named(
                "time",
                Some(map! { "xmlns" => namespaces::NS_URN_TIME }),
                Some(vec![
                    ProseProtocolStanza::named(
                        "tzo",
                        None,
                        Some(vec![ProseProtocolStanza::text("-06:00")?]),
                    )?,
                    ProseProtocolStanza::named(
                        "utc",
                        None,
                        Some(vec![ProseProtocolStanza::text("2006-12-19T17:58:35Z")?]),
                    )?,
                ]),
            )?]),
        )?);

        Ok(())
    }

    fn handle_get_ping(connection: &mut Connection, stanza: &Stanza) -> Result<(), Error> {
        // @ref: https://xmpp.org/extensions/xep-0199.html

        connection.send(&ProseProtocolStanza::result(stanza, None)?);

        Ok(())
    }
}
