// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::account::{AccountObserver, IDProvider};
use crate::connection::{ConnectionEvent, XMPPConnection, XMPPSender};
use crate::error::Result;
use crate::extensions::{Chat, Debug, Presence, Profile, Roster, MAM};
use crate::extensions::{XMPPExtension, XMPPExtensionContext};
use crate::helpers::StanzaExt;
use crate::types::namespace::Namespace;
use jid::{BareJid, FullJid};
use libstrophe::{Stanza, StanzaRef};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

pub struct Account {
    _ctx: Arc<XMPPExtensionContext>,
    pub roster: Arc<Roster>,
    pub chat: Arc<Chat>,
    pub presence: Arc<Presence>,
    pub mam: Arc<MAM>,
    pub profile: Arc<Profile>,
    pub debug: Arc<Debug>,
}

impl Account {
    pub fn new(
        jid: &FullJid,
        connection: Box<dyn XMPPConnection>,
        id_provider: Box<dyn IDProvider>,
        observer: Box<dyn AccountObserver>,
    ) -> Result<Account> {
        let mut connection = connection;

        let ctx = Arc::new(XMPPExtensionContext::new(
            jid.clone(),
            Box::new(PlaceholderSender { sender: None }),
            id_provider,
            observer,
        ));

        let roster = Arc::new(Roster::new(ctx.clone()));
        let chat = Arc::new(Chat::new(ctx.clone()));
        let presence = Arc::new(Presence::new(ctx.clone()));
        let mam = Arc::new(MAM::new(ctx.clone()));
        let profile = Arc::new(Profile::new(ctx.clone()));
        let debug = Arc::new(Debug::new(ctx.clone()));

        let extensions: Vec<Arc<dyn XMPPExtension>> = vec![
            presence.clone(),
            roster.clone(),
            chat.clone(),
            mam.clone(),
            profile.clone(),
            debug.clone(),
        ];

        fn for_each<F: Fn(&Arc<dyn XMPPExtension>) -> Result<()>>(
            extensions: &Vec<Arc<dyn XMPPExtension>>,
            handler: F,
        ) {
            for extension in extensions {
                match handler(&extension) {
                    Ok(_) => {}
                    Err(error) => log::error!("{:?}", error),
                }
            }
        }

        {
            let extensions = extensions.clone();
            let ctx = ctx.clone();
            connection.set_connection_handler(Box::new(
                move |event: &ConnectionEvent| match event {
                    ConnectionEvent::Connect => {
                        for_each(&extensions, |e| e.handle_connect());
                        ctx.observer.did_connect();
                    }
                    ConnectionEvent::Disconnect(_) => {
                        for_each(&extensions, |e| e.handle_disconnect());
                        ctx.observer.did_disconnect();
                    }
                },
            ));
        }

        {
            let ctx = ctx.clone();
            connection.set_stanza_handler(Box::new(move |stanza: &Stanza| {
                let name = match stanza.name() {
                    Some(name) => name,
                    None => return,
                };

                match name {
                    "presence" => for_each(&extensions, |e| e.handle_presence_stanza(stanza)),
                    "message" => {
                        if let Some(event) =
                            stanza.get_child_by_name_and_ns("event", Namespace::PubSubEvent)
                        {
                            let from = match stanza.from().and_then(|a| BareJid::from_str(a).ok()) {
                                Some(from) => from,
                                None => {
                                    log::error!("Missing sender in pubsub event.");
                                    return;
                                }
                            };

                            let items = match event.get_child_by_name("items") {
                                Some(items) => items,
                                None => {
                                    log::error!("Missing items node in pubsub event.");
                                    return;
                                }
                            };

                            let node = match items.get_attribute("node") {
                                Some(node) => node,
                                None => {
                                    log::error!("Missing node attribute in pubsub event.");
                                    return;
                                }
                            };

                            for_each(&extensions, |e| {
                                e.handle_pubsub_event(&from, node, items.deref())
                            });

                            return;
                        }

                        for_each(&extensions, |e| e.handle_message_stanza(stanza))
                    }
                    "iq" => {
                        for_each(&extensions, |e| e.handle_iq_stanza(stanza));

                        let payload: Option<StanzaRef>;
                        let is_error: bool;

                        match stanza.get_attribute("type") {
                            Some("result") => {
                                payload = stanza.get_first_non_text_child();
                                is_error = false
                            }
                            Some("error") => {
                                payload = stanza.get_child_by_name("error");
                                is_error = true
                            }
                            _ => return,
                        };

                        let (id, payload) = match (stanza.id(), payload) {
                            (Some(id), Some(payload)) => (id, payload),
                            (_, _) => return,
                        };

                        let result = if is_error { Err(payload) } else { Ok(payload) };

                        if let Some(err) = ctx.handle_iq_result(id, result).err() {
                            log::error!("{:?}", err);
                        }
                    }
                    _ => (),
                }
            }));
        }

        let sender = connection.connect()?;
        ctx.replace_sender(sender)?;

        Ok(Account {
            _ctx: ctx,
            presence,
            roster,
            chat,
            mam,
            profile,
            debug,
        })
    }
}

struct PlaceholderSender {
    sender: Option<Box<dyn XMPPSender>>,
}

impl XMPPSender for PlaceholderSender {
    fn send_stanza(&self, stanza: Stanza) -> Result<()> {
        if let Some(sender) = &self.sender {
            return sender.send_stanza(stanza);
        }
        return Ok(());
    }
}
