// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::VecDeque;
use std::sync::Arc;

use minidom::Element;
use parking_lot::Mutex;

use prose_core_client::ClientEvent;

use super::element_ext::ElementExt;

#[derive(Debug)]
struct Message {
    file: String,
    line: u32,
    r#type: MessageType,
}

#[derive(Debug)]
pub enum MessageType {
    In(Element),
    Out(Element),
    Event(ClientEvent),
}

#[derive(Default, Clone)]
pub struct TestMessageQueue {
    messages: Arc<Mutex<VecDeque<Message>>>,
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
            r#type: MessageType::Event(event),
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

    pub fn pop_event(&self) -> Option<(ClientEvent, String, u32)> {
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
