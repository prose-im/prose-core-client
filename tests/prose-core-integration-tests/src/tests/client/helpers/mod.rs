// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub(self) use element_ext::ElementExt;
pub use test_client::TestClient;
pub(self) use test_message_queue::TestMessageQueue;

mod connector;
mod element_ext;
mod test_client;
mod test_client_login;
mod test_client_muc;
mod test_message_queue;
