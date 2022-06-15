use libstrophe::{Connection, Stanza};
use std::sync::mpsc::{channel, Sender, TryRecvError};

use std::io;
use std::io::prelude::*;
use std::thread::{self, JoinHandle};
use std::time::Duration;

struct Message {
    text: String,
    who: String,
}

struct Chat {
    jid: String,
    message_channel: Sender<Message>,
    _thread: JoinHandle<()>,
}

impl Chat {
    fn new(jid: &str, password: &str, other_jid: &str) -> Chat {
        let (tx, rx) = channel::<Message>();

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

            println!(
                "{} says: {}",
                stanza.from().expect("Cannot get from"),
                body.text().expect("Cannot get body")
            );
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

        let send_handler = move |_ctx: &libstrophe::Context, conn: &mut Connection| {
            match rx.try_recv() {
                Ok(msg) => {
                    print!("Sending message {} to {}", msg.text, msg.who);
                    let mut stanza = Stanza::new_message(Some("chat"), None, Some(&msg.who));
                    stanza.set_body(&msg.text).unwrap();
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
        conn.set_jid(jid);
        conn.set_pass(password);
        conn.set_flags(libstrophe::ConnectionFlags::TRUST_TLS)
            .unwrap();

        let ctx = conn
            .connect_client(None, None, conn_handler)
            .expect("Cannot connect to XMPP server");
        let thread = Some(thread::Builder::new().spawn(move || ctx.run()).unwrap());

        Chat {
            jid: other_jid.to_string(),
            message_channel: tx,
            _thread: thread.unwrap(),
        }
    }

    fn send_message(&self, message: String) {
        let msg = Message {
            text: message,
            who: self.jid.clone(),
        };
        self.message_channel.send(msg).unwrap();
    }
}

enum InputState {
    Jid,
    Password { jid: String },
    OtherJid { jid: String, password: String },
    Message { chat: Chat },
}

fn main() {
    let mut state = InputState::Jid;
    let stdin = io::stdin();

    println!("Enter your jid: ");
    for line in stdin.lock().lines() {
        match line {
            Ok(ref line) => match state {
                InputState::Jid => {
                    state = InputState::Password { jid: line.clone() };
                    println!("Enter password: ");
                }
                InputState::Password { jid } => {
                    state = InputState::OtherJid {
                        jid: jid,
                        password: line.clone(),
                    };
                    println!("Enter their jid: ");
                }
                InputState::OtherJid {
                    ref jid,
                    ref password,
                } => {
                    state = InputState::Message {
                        chat: Chat::new(jid, password, line),
                    };
                    println!("Enter message: ");
                }
                InputState::Message { ref chat } => chat.send_message(line.clone()),
            },
            Err(err) => {
                print!("Error: {}", err);
                break;
            }
        }
    }
}
