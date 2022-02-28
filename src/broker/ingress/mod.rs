// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use std::sync::Arc;

use futures::stream::StreamExt;
use tokio_xmpp::AsyncClient as XMPPClient;
use xmpp_parsers::message::{Body, Message, MessageType};
use xmpp_parsers::presence::{Presence, Show as PresenceShow, Type as PresenceType};
use xmpp_parsers::{Element, Jid};

use super::ProseBrokerClient;

// -- Structures --

pub struct ProseBrokerIngress {
    client: ProseBrokerClient,
}

// -- Implementations --

impl ProseBrokerIngress {
    pub fn new(client: ProseBrokerClient) -> Self {
        ProseBrokerIngress { client: client }
    }

    pub async fn listen(&self) {
        // TODO: refactor this, as this is just copy & paste from tokio-xmpp examples

        // Main loop, processes events
        let mut wait_for_stream_end = false;
        let mut stream_ended = false;

        while !stream_ended {
            let mut client = self.client.write().unwrap();

            if let Some(event) = client.next().await {
                log::debug!("[broker] event: {:?}", event);

                if wait_for_stream_end {
                    /* Do nothing */
                } else if event.is_online() {
                    let jid = event
                        .get_jid()
                        .map(|jid| format!("{}", jid))
                        .unwrap_or("unknown".to_owned());

                    log::debug!("[broker] online at {}", jid);

                    let mut presence = Presence::new(PresenceType::None);
                    presence.show = Some(PresenceShow::Chat);
                    presence
                        .statuses
                        .insert(String::from("en"), String::from("Echoing messages."));

                    client.send_stanza(presence.into()).await.unwrap();
                } else if let Some(message) = event
                    .into_stanza()
                    .and_then(|stanza| Message::try_from(stanza).ok())
                {
                    match (message.from, message.bodies.get("")) {
                        (Some(ref from), Some(ref body)) if body.0 == "die" => {
                            log::debug!("[broker] secret die command triggered by {}", from);

                            wait_for_stream_end = true;

                            client.send_end().await.unwrap();
                        }
                        (Some(ref from), Some(ref body)) => {
                            if message.type_ != MessageType::Error {
                                // This is a message we'll echo
                                let mut message = Message::new(Some(from.clone()));
                                message
                                    .bodies
                                    .insert(String::new(), Body(body.0.to_owned()));

                                client.send_stanza(message.into()).await.unwrap();
                            }
                        }
                        _ => {}
                    }
                }
            } else {
                log::debug!("[broker] stream_ended");

                stream_ended = true;
            }
        }
    }
}
