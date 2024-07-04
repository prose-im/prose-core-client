// prose-core-client/prose-core-integration-tests
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_core_client::{Client, ClientDelegate, ClientEvent};

use crate::tests::client::helpers::element_ext::ElementExt;
use crate::tests::client::helpers::test_message_queue::{MessageType, TestMessageQueue};

pub struct Delegate {
    messages: TestMessageQueue,
}

impl Delegate {
    pub fn new(messages: TestMessageQueue) -> Self {
        Self { messages }
    }
}

impl ClientDelegate for Delegate {
    fn handle_event(&self, _client: Client, received_event: ClientEvent) {
        let Some((matcher, file, line)) = self.messages.pop_event() else {
            let mut panic_message =
                format!("\nClient sent unexpected event:\n\n{:?}", received_event);

            if let Some((message, file, line)) = self.messages.pop_message() {
                let element = match message {
                    MessageType::In(elem) => elem
                        .to_pretty_printed_xml()
                        .expect("Failed to convert cached stanza to XML"),
                    MessageType::Out(elem) => elem
                        .to_pretty_printed_xml()
                        .expect("Failed to convert cached stanza to XML"),
                    MessageType::Event(_) => unreachable!(),
                };

                panic_message.push_str(&format!(
                    "\n\nNext expected message is:\n\n{element}\n\nScheduled at:\n{file}:{line}\n",
                ))
            } else {
                panic_message.push_str("\n\nThere were no further messages scheduled.")
            }

            panic!("{}", panic_message);
        };

        matcher.assert_event(received_event, file, line);
    }
}
