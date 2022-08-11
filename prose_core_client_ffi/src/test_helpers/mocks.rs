use crate::account::IDProvider;
use crate::{
    Account, AccountObserverMock, ConnectionEvent, ConnectionHandler, Result, StanzaHandler,
    XMPPConnection, XMPPSender,
};
use jid::FullJid;
use libstrophe::Stanza;
use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

impl Account {
    pub fn connected() -> (
        Self,
        Rc<RefCell<HandlerBucket>>,
        Rc<StanzaBucket>,
        Arc<Mutex<AccountObserverMock<'static>>>,
    ) {
        let handlers = HandlerBucket::new();
        let stanzas = StanzaBucket::new();
        let observer = Arc::new(Mutex::new(AccountObserverMock::new()));

        // We start our MockIDProvider with a negative value to compensate for requests that
        // happen immediately after the connection was established.
        let account = Account::new(
            &FullJid::from_str("test@prose.org/ci").unwrap(),
            MockConnection::new(handlers.clone(), stanzas.clone()),
            MockIDProvider::new(-1),
            Box::new(observer.clone()),
        )
        .unwrap();

        observer
            .lock()
            .unwrap()
            .expect_did_connect()
            .times(1)
            .returns(());

        handlers.send_connection_event(ConnectionEvent::Connect);

        stanzas.clear();

        (account, handlers, stanzas, observer)
    }
}

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
}

pub trait HandlerBucketExt {
    fn send_connection_event(&self, event: ConnectionEvent);
    fn send_stanza_str(&self, stanza_str: &str);
}

impl HandlerBucketExt for Rc<RefCell<HandlerBucket>> {
    fn send_connection_event(&self, event: ConnectionEvent) {
        let mut bucket = self.deref().borrow_mut();
        let handler = bucket
            .deref_mut()
            .connection_handler
            .as_mut()
            .expect("Connection Handler not set.");
        handler(&event);
    }

    fn send_stanza_str(&self, stanza_str: &str) {
        let mut bucket = self.deref().borrow_mut();
        let handler = bucket
            .deref_mut()
            .stanza_handler
            .as_mut()
            .expect("Stanza Handler not set.");
        handler(&Stanza::from_str(stanza_str));
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

    pub fn clear(&self) {
        self.stanzas.borrow_mut().clear();
    }

    pub fn stanza_at_index(&self, index: usize) -> Stanza {
        self.stanzas.borrow()[index].clone()
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
        let mut bucket = self.handler_bucket.deref().borrow_mut();
        bucket.connection_handler = Some(handler);
    }

    fn set_stanza_handler(&mut self, handler: StanzaHandler) {
        let mut bucket = self.handler_bucket.deref().borrow_mut();
        bucket.stanza_handler = Some(handler);
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

pub struct MockIDProvider {
    last_id: Rc<Cell<i64>>,
}

impl MockIDProvider {
    pub fn new(start_index: i64) -> Box<Self> {
        Box::new(MockIDProvider {
            last_id: Rc::new(Cell::new(start_index)),
        })
    }
}

impl IDProvider for MockIDProvider {
    fn new_id(&self) -> String {
        self.last_id.set(self.last_id.get() + 1);
        format!("id_{}", self.last_id.get())
    }
}

unsafe impl Send for MockIDProvider {}
unsafe impl Sync for MockIDProvider {}
