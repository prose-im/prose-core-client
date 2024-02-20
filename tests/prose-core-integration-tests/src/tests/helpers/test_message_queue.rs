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
enum Message {
    In(Element),
    Out(Element),
    Event(ClientEvent),
}

#[derive(Default, Clone)]
pub struct TestMessageQueue {
    messages: Arc<Mutex<VecDeque<Message>>>,
}

impl TestMessageQueue {
    pub fn send(&self, xml: impl Into<String>) {
        self.expect_send_element(
            // minidom is super particular when it comes to whitespaces,
            // so we'll format the string first…
            Element::from_pretty_printed_xml(&xml.into()).expect("Failed to parse xml"),
        )
    }

    pub fn expect_send_element(&self, element: impl Into<Element>) {
        self.messages.lock().push_back(Message::Out(element.into()))
    }

    pub fn receive(&self, xml: impl Into<String>) {
        self.receive_element(
            Element::from_pretty_printed_xml(&xml.into()).expect("Failed to parse xml"),
        )
    }

    pub fn receive_element(&self, element: impl Into<Element>) {
        self.messages.lock().push_back(Message::In(element.into()))
    }

    pub fn event(&self, event: ClientEvent) {
        self.messages.lock().push_back(Message::Event(event))
    }
}

impl TestMessageQueue {
    pub fn pop_send(&self) -> Option<Element> {
        let mut guard = self.messages.lock();
        match guard.pop_front() {
            None => None,
            Some(Message::In(element)) => {
                guard.push_front(Message::In(element));
                None
            }
            Some(Message::Out(element)) => Some(element),
            Some(Message::Event(event)) => {
                guard.push_front(Message::Event(event));
                None
            }
        }
    }

    pub fn pop_receive(&self) -> Option<Element> {
        let mut guard = self.messages.lock();
        match guard.pop_front() {
            None => None,
            Some(Message::In(element)) => Some(element),
            Some(Message::Out(element)) => {
                guard.push_front(Message::Out(element));
                None
            }
            Some(Message::Event(event)) => {
                guard.push_front(Message::Event(event));
                None
            }
        }
    }

    pub fn pop_event(&self) -> Option<ClientEvent> {
        let mut guard = self.messages.lock();
        match guard.pop_front() {
            None => None,
            Some(Message::In(element)) => {
                guard.push_front(Message::In(element));
                None
            }
            Some(Message::Out(element)) => {
                guard.push_front(Message::Out(element));
                None
            }
            Some(Message::Event(event)) => Some(event),
        }
    }
}
