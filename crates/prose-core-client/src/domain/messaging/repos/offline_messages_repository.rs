// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::app::event_handlers::MessageEvent;

#[cfg_attr(feature = "test", mockall::automock)]
pub trait OfflineMessagesRepository: SendUnlessWasm + SyncUnlessWasm {
    fn push(&self, event: MessageEvent);
    fn drain(&self) -> Vec<MessageEvent>;
}
