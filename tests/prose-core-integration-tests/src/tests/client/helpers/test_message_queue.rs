// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use minidom::Element;
use parking_lot::Mutex;
use pretty_assertions::assert_eq;

use prose_core_client::dtos::RoomId;
use prose_core_client::{ClientEvent, ClientRoomEventType};

use super::element_ext::ElementExt;

struct Message {
    file: String,
    line: u32,
    r#type: MessageType,
}

pub enum MessageType {
    In(Element),
    Out(Element),
    Event(ClientEventMatcher),
}

pub struct ClientEventMatcher {
    matcher: Box<dyn FnOnce(ClientEvent, String, u32) + Send>,
}

impl Debug for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}\n\n{}:{}\n", self.r#type, self.file, self.line)
    }
}

impl Debug for MessageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::In(element) => write!(
                f,
                "[Receive]\n{}",
                element
                    .to_pretty_printed_xml()
                    .expect("Failed to format element")
            ),
            MessageType::Out(element) => write!(
                f,
                "[Send]\n{}",
                element
                    .to_pretty_printed_xml()
                    .expect("Failed to format element")
            ),
            MessageType::Event(matcher) => {
                write!(f, "[Event]\n")?;
                matcher.fmt(f)
            }
        }
    }
}

impl Debug for ClientEventMatcher {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClientEvent")
    }
}

impl ClientEventMatcher {
    pub fn event(expected_event: ClientEvent) -> Self {
        Self {
            matcher: Box::new(move |event, file, line| {
                assert_eq!(
                    expected_event, event,
                    "\n\n➡️ Assertion failed at:\n{}:{}",
                    file, line
                )
            }),
        }
    }

    pub fn room_event(room_id: RoomId, expected_type: ClientRoomEventType) -> Self {
        Self {
            matcher: Box::new(move |event, file, line| {
                let ClientEvent::RoomChanged {
                    room,
                    r#type: event_type,
                } = event
                else {
                    panic!("Expected to receive a ClientEvent::RoomChanged. Received: \n{:?}\n\n➡️ Assertion failed at:\n{}:{}", event, file, line);
                };

                assert_eq!(
                    &room_id,
                    room.to_generic_room().jid(),
                    "\n\n➡️ Assertion failed at:\n{}:{}",
                    file,
                    line
                );

                assert_eq!(
                    expected_type, event_type,
                    "\n\n➡️ Assertion failed at:\n{}:{}",
                    file, line
                )
            }),
        }
    }

    pub fn any() -> Self {
        Self {
            matcher: Box::new(|_, _, _| {}),
        }
    }

    pub fn assert_event(self, event: ClientEvent, file: String, line: u32) {
        (self.matcher)(event, file, line)
    }
}

#[derive(Default, Clone)]
pub struct TestMessageQueue {
    messages: Arc<Mutex<VecDeque<Message>>>,
}

impl Debug for TestMessageQueue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for message in self.messages.lock().iter() {
            write!(f, "\n{:?}", message)?;
        }
        Ok(())
    }
}

impl TestMessageQueue {
    pub fn send(&self, xml: impl Into<String>, file: &str, line: u32) {
        self.expect_send_element(
            // minidom is super particular when it comes to whitespaces,
            // so we'll format the string first…
            Element::from_pretty_printed_xml(&xml.into()).expect("Failed to parse xml"),
            file,
            line,
        )
    }

    pub fn expect_send_element(&self, element: impl Into<Element>, file: &str, line: u32) {
        self.messages.lock().push_back(Message {
            file: file.to_string(),
            line,
            r#type: MessageType::Out(element.into()),
        })
    }

    pub fn receive(&self, xml: impl Into<String>, file: &str, line: u32) {
        self.receive_element(
            Element::from_pretty_printed_xml(&xml.into()).expect("Failed to parse xml"),
            file,
            line,
        )
    }

    pub fn receive_element(&self, element: impl Into<Element>, file: &str, line: u32) {
        self.messages.lock().push_back(Message {
            file: file.to_string(),
            line,
            r#type: MessageType::In(element.into()),
        })
    }

    pub fn event(&self, event: ClientEvent, file: &str, line: u32) {
        self.messages.lock().push_back(Message {
            file: file.to_string(),
            line,
            r#type: MessageType::Event(ClientEventMatcher::event(event)),
        })
    }

    pub fn room_event(
        &self,
        room_id: RoomId,
        event_type: ClientRoomEventType,
        file: &str,
        line: u32,
    ) {
        self.messages.lock().push_back(Message {
            file: file.to_string(),
            line,
            r#type: MessageType::Event(ClientEventMatcher::room_event(room_id, event_type)),
        })
    }

    pub fn any_event(&self, file: &str, line: u32) {
        self.messages.lock().push_back(Message {
            file: file.to_string(),
            line,
            r#type: MessageType::Event(ClientEventMatcher::any()),
        })
    }

    pub fn len(&self) -> usize {
        self.messages.lock().len()
    }
}

impl TestMessageQueue {
    pub fn pop_send(&self) -> Option<(Element, String, u32)> {
        let mut guard = self.messages.lock();

        let Some(message) = guard.pop_front() else {
            return None;
        };

        match message.r#type {
            MessageType::In(element) => {
                guard.push_front(Message {
                    file: message.file,
                    line: message.line,
                    r#type: MessageType::In(element),
                });
                None
            }
            MessageType::Out(element) => Some((element, message.file, message.line)),
            MessageType::Event(event) => {
                guard.push_front(Message {
                    file: message.file,
                    line: message.line,
                    r#type: MessageType::Event(event),
                });
                None
            }
        }
    }

    pub fn pop_receive(&self) -> Option<Element> {
        let mut guard = self.messages.lock();

        let Some(message) = guard.pop_front() else {
            return None;
        };

        match message.r#type {
            MessageType::In(element) => Some(element),
            MessageType::Out(element) => {
                guard.push_front(Message {
                    file: message.file,
                    line: message.line,
                    r#type: MessageType::Out(element),
                });
                None
            }
            MessageType::Event(event) => {
                guard.push_front(Message {
                    file: message.file,
                    line: message.line,
                    r#type: MessageType::Event(event),
                });
                None
            }
        }
    }

    pub fn pop_event(&self) -> Option<(ClientEventMatcher, String, u32)> {
        let mut guard = self.messages.lock();

        let Some(message) = guard.pop_front() else {
            return None;
        };

        match message.r#type {
            MessageType::In(element) => {
                guard.push_front(Message {
                    file: message.file,
                    line: message.line,
                    r#type: MessageType::In(element),
                });
                None
            }
            MessageType::Out(element) => {
                guard.push_front(Message {
                    file: message.file,
                    line: message.line,
                    r#type: MessageType::Out(element),
                });
                None
            }
            MessageType::Event(event) => Some((event, message.file, message.line)),
        }
    }

    pub fn pop_message(&self) -> Option<(MessageType, String, u32)> {
        let Some(message) = self.messages.lock().pop_front() else {
            return None;
        };
        Some((message.r#type, message.file, message.line))
    }
}
