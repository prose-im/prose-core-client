use crate::util::XMPPElement;
use std::task::Waker;

pub(crate) enum ModuleFuturePoll {
    Pending,
    Ready(Option<Waker>),
}

pub(crate) trait ModuleFutureState: Send {
    fn handle_element(&mut self, element: &XMPPElement) -> ModuleFuturePoll;
    fn fail_with_timeout(&mut self) -> Option<Waker>;
}
