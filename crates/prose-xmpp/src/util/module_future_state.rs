// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::util::XMPPElement;
use std::task::Waker;

pub(crate) enum ModuleFuturePoll {
    Pending(Option<XMPPElement>),
    Ready(Option<Waker>),
}

pub(crate) trait ModuleFutureState: Send {
    fn handle_element(&mut self, element: XMPPElement) -> ModuleFuturePoll;
    fn fail_with_timeout(&mut self) -> Option<Waker>;
    fn fail_with_disconnect(&mut self) -> Option<Waker>;
}
