use crate::account::{AccountObserver, IDProvider};
use crate::connection::{ConnectionEvent, XMPPConnection, XMPPSender};
use crate::error::Result;
use crate::extensions::{Chat, Debug, Presence, Roster};
use crate::extensions::{XMPPExtension, XMPPExtensionContext};
use libstrophe::Stanza;
use std::sync::Arc;

pub struct Account {
    _ctx: Arc<XMPPExtensionContext>,
    pub roster: Arc<Roster>,
    pub chat: Arc<Chat>,
    pub presence: Arc<Presence>,
    pub debug: Arc<Debug>,
}

impl Account {
    pub fn new(
        connection: Box<dyn XMPPConnection>,
        id_provider: Box<dyn IDProvider>,
        observer: Box<dyn AccountObserver>,
    ) -> Result<Account> {
        let mut connection = connection;

        let ctx = Arc::new(XMPPExtensionContext::new(
            Box::new(PlaceholderSender { sender: None }),
            id_provider,
            observer,
        ));

        let roster = Arc::new(Roster::new(ctx.clone()));
        let chat = Arc::new(Chat::new(ctx.clone()));
        let presence = Arc::new(Presence::new(ctx.clone()));
        let debug = Arc::new(Debug::new(ctx.clone()));

        let extensions: Vec<Arc<dyn XMPPExtension>> = vec![
            roster.clone(),
            chat.clone(),
            presence.clone(),
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

        let ec = extensions.clone();
        let ctxc = ctx.clone();
        connection.set_connection_handler(Box::new(move |event: &ConnectionEvent| match event {
            ConnectionEvent::Connect => {
                for_each(&ec, |e| e.handle_connect());
                ctxc.observer.did_connect();
            }
            ConnectionEvent::Disconnect(_) => {
                for_each(&ec, |e| e.handle_disconnect());
                ctxc.observer.did_disconnect();
            }
        }));

        connection.set_stanza_handler(Box::new(move |stanza: &Stanza| {
            let name = match stanza.name() {
                Some(name) => name,
                None => return,
            };

            match name {
                "presence" => for_each(&extensions, |e| e.handle_presence_stanza(stanza)),
                "message" => for_each(&extensions, |e| e.handle_message_stanza(stanza)),
                "iq" => for_each(&extensions, |e| e.handle_iq_stanza(stanza)),
                _ => (),
            }
        }));

        let sender = connection.connect()?;
        ctx.replace_sender(sender)?;

        Ok(Account {
            _ctx: ctx,
            roster,
            chat,
            presence,
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
