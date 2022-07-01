use crate::{
    AccountObserver, ConnectionEvent, ConnectionHandler, Message, Presence, Result, Roster,
    StanzaHandler, XMPPConnection, XMPPSender,
};
use libstrophe::Stanza;
use std::cell::RefCell;
use std::rc::Rc;

pub struct HandlerBucket {
    connection_handler: Option<ConnectionHandler>,
    stanza_handler: Option<StanzaHandler>,
}

impl HandlerBucket {
    pub fn new() -> Rc<RefCell<HandlerBucket>> {
        Rc::new(RefCell::new(HandlerBucket {
            connection_handler: None,
            stanza_handler: None,
        }))
    }

    pub fn send_connection_event(&mut self, event: ConnectionEvent) {
        let handler = self
            .connection_handler
            .as_mut()
            .expect("Connection Handler not set.");
        handler(&event);
    }

    pub fn send_stanza_str(&mut self, stanza_str: &str) {
        let handler = self
            .stanza_handler
            .as_mut()
            .expect("Stanza Handler not set.");
        let stanza = Stanza::from_str(stanza_str);
        handler(&stanza);
    }
}

pub struct StanzaBucket {
    pub stanzas: RefCell<Vec<Stanza>>,
}

impl StanzaBucket {
    pub fn new() -> Rc<StanzaBucket> {
        Rc::new(StanzaBucket {
            stanzas: RefCell::new(vec![]),
        })
    }
}

pub struct MockConnection {
    handler_bucket: Rc<RefCell<HandlerBucket>>,
    stanza_bucket: Rc<StanzaBucket>,
}

impl MockConnection {
    pub fn new(
        handler_bucket: Rc<RefCell<HandlerBucket>>,
        stanza_bucket: Rc<StanzaBucket>,
    ) -> Box<Self> {
        Box::new(MockConnection {
            handler_bucket,
            stanza_bucket,
        })
    }
}

impl XMPPConnection for MockConnection {
    fn connect(self: Box<Self>) -> Result<Box<dyn XMPPSender>> {
        Ok(Box::new(MockSender {
            stanza_bucket: self.stanza_bucket,
        }))
    }

    fn set_connection_handler(&mut self, handler: ConnectionHandler) {
        self.handler_bucket.borrow_mut().connection_handler = Some(handler);
    }

    fn set_stanza_handler(&mut self, handler: StanzaHandler) {
        self.handler_bucket.borrow_mut().stanza_handler = Some(handler);
    }
}

struct MockSender {
    stanza_bucket: Rc<StanzaBucket>,
}

impl XMPPSender for MockSender {
    fn send_stanza(&self, stanza: Stanza) -> Result<()> {
        self.stanza_bucket.stanzas.borrow_mut().push(stanza);
        Ok(())
    }
}
unsafe impl Send for MockSender {}

pub struct MockAccountObserver {}

impl MockAccountObserver {
    pub fn new() -> Box<Self> {
        Box::new(MockAccountObserver {})
    }
}

#[allow(non_snake_case)]
impl AccountObserver for MockAccountObserver {
    fn didConnect(&self) {}
    fn didDisconnect(&self) {}
    fn didReceiveMessage(&self, _message: Message) {}
    fn didReceiveRoster(&self, _roster: Roster) {}
    fn didReceivePresence(&self, _presence: Presence) {}
}
