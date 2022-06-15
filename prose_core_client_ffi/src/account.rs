use super::AccountObserver;
use super::Message;
use jid::BareJid;
use libstrophe::{Connection, Stanza};
use std::str::FromStr;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Account {
    observer: Arc<Box<dyn AccountObserver>>,
    message_channel: Sender<Message>,
    thread: JoinHandle<()>,
}

impl Account {
    pub fn new(jid: &BareJid, password: &str, observer: Arc<Box<dyn AccountObserver>>) -> Account {
        let (tx, rx) = channel::<Message>();
        let local_observer = observer.clone();

        let message_handler = move |_ctx: &libstrophe::Context,
                                    _conn: &mut libstrophe::Connection,
                                    stanza: &libstrophe::Stanza| {
            let body = match stanza.get_child_by_name("body") {
                Some(body) => body,
                None => return true,
            };

            match stanza.stanza_type() {
                Some(typ) => {
                    if typ == "error" {
                        return true;
                    }
                }
                None => return true,
            };

            local_observer.as_ref().didReceive(Message {
                from: BareJid::from_str(stanza.from().expect("Cannot get from"))
                    .expect("Cannot parse JID"),
                body: body.text().expect("Cannot get body"),
            });
            true
        };

        let presence_handler = move |_ctx: &libstrophe::Context,
                                     _conn: &mut libstrophe::Connection,
                                     stanza: &libstrophe::Stanza| {
            let status = match stanza.get_child_by_name("show") {
                Some(show) => show,
                None => return true,
            };

            match stanza.stanza_type() {
                Some(typ) => {
                    if typ == "error" {
                        return true;
                    }
                }
                None => return true,
            };

            println!(
                "{} updated presence to {}",
                stanza.from().expect("Cannot get from"),
                status.text().expect("Cannot get show")
            );
            true
        };

        let conn_handler = |ctx: &libstrophe::Context,
                            conn: &mut libstrophe::Connection,
                            evt: libstrophe::ConnectionEvent| {
            match evt {
                libstrophe::ConnectionEvent::Connect => {
                    println!("Connected");
                    let pres = libstrophe::Stanza::new_presence();
                    conn.send(&pres);
                }
                libstrophe::ConnectionEvent::Disconnect(err) => {
                    println!("Disconnected, Reason: {:?}", err);
                    ctx.stop();
                }
                _ => unimplemented!(),
            }
        };

        let send_handler = move |ctx: &libstrophe::Context, conn: &mut Connection| {
            match rx.try_recv() {
                Ok(msg) => {
                    print!("Sending message {} to {}", msg.body, msg.from.to_string());
                    let mut stanza =
                        Stanza::new_message(Some("chat"), None, Some(&msg.from.to_string()));
                    stanza.set_body(&msg.body.to_string()).unwrap();
                    conn.send(&stanza)
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => return false,
            }
            true
        };

        let mut conn = libstrophe::Connection::new(libstrophe::Context::new_with_default_logger());
        conn.set_flags(libstrophe::ConnectionFlags::TRUST_TLS)
            .unwrap();
        conn.handler_add(message_handler, None, Some("message"), None)
            .unwrap();
        conn.handler_add(presence_handler, None, Some("presence"), None)
            .unwrap();
        conn.timed_handler_add(send_handler, Duration::from_millis(1))
            .unwrap();
        conn.set_jid(jid.to_string());
        conn.set_pass(password);
        conn.set_flags(libstrophe::ConnectionFlags::TRUST_TLS)
            .unwrap();

        let ctx = conn
            .connect_client(None, None, conn_handler)
            .expect("Cannot connect to XMPP server");
        let thread = Some(thread::Builder::new().spawn(move || ctx.run()).unwrap());

        Account {
            observer: observer,
            message_channel: tx,
            thread: thread.unwrap(),
        }
    }

    pub fn send_message(&self, jid: &BareJid, body: &str) {
        let msg = Message {
            from: jid.clone(),
            body: body.to_string(),
        };
        self.message_channel.send(msg).unwrap();
    }
}
