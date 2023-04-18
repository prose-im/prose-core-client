use std::ops;
use std::ops::Deref;

pub(crate) enum StanzaCow<'a> {
    Stanza(libstrophe::Stanza),
    StanzaBorrow(&'a libstrophe::Stanza),
    StanzaRef(libstrophe::StanzaRef<'a>),
}

// N.B.: StanzaRef isn't `Send` but to the best of my knowledge _should_ be. Stanza itself is `Send`
// and StanzaRef only contains a non-owning Stanza (basically a ref) which doesn't release the
// FFI pointer on drop. So we should be safe? This is required so that our RequestPromiseState
// which is held by Client and will be moved between the main thread (or from whatever thread the
// SDK consumer calls us) and the LibstropheConnector thread.
unsafe impl<'a> Send for StanzaCow<'a> {}

impl Clone for StanzaCow<'_> {
    fn clone(&self) -> Self {
        StanzaCow::Stanza(self.deref().clone())
    }
}

impl<'a> ops::Deref for StanzaCow<'a> {
    type Target = libstrophe::Stanza;

    fn deref(&self) -> &Self::Target {
        match self {
            StanzaCow::Stanza(stanza) => stanza,
            StanzaCow::StanzaBorrow(stanza) => *stanza,
            StanzaCow::StanzaRef(stanza_ref) => &stanza_ref,
        }
    }
}

impl<'a> StanzaCow<'a> {
    pub fn to_mut(&mut self) -> &mut libstrophe::Stanza {
        match self {
            StanzaCow::StanzaBorrow(stanza) => {
                *self = Self::Stanza(stanza.to_owned());
                match *self {
                    StanzaCow::StanzaBorrow(..) => unreachable!(),
                    StanzaCow::StanzaRef(..) => unreachable!(),
                    StanzaCow::Stanza(ref mut stanza) => stanza,
                }
            }
            StanzaCow::Stanza(ref mut stanza) => stanza,
            StanzaCow::StanzaRef(stanza) => {
                *self = Self::Stanza(stanza.to_owned());
                match *self {
                    StanzaCow::StanzaBorrow(..) => unreachable!(),
                    StanzaCow::StanzaRef(..) => unreachable!(),
                    StanzaCow::Stanza(ref mut stanza) => stanza,
                }
            }
        }
    }

    pub fn into_owned(self) -> libstrophe::Stanza {
        match self {
            StanzaCow::Stanza(stanza) => stanza,
            StanzaCow::StanzaBorrow(stanza) => stanza.clone(),
            StanzaCow::StanzaRef(stanza) => stanza.to_owned(),
        }
    }
}

impl<'a> From<libstrophe::Stanza> for StanzaCow<'a> {
    fn from(value: libstrophe::Stanza) -> Self {
        StanzaCow::Stanza(value)
    }
}

impl<'a> From<&'a libstrophe::Stanza> for StanzaCow<'a> {
    fn from(value: &'a libstrophe::Stanza) -> Self {
        StanzaCow::StanzaBorrow(value)
    }
}

impl<'a> From<libstrophe::StanzaRef<'a>> for StanzaCow<'a> {
    fn from(value: libstrophe::StanzaRef<'a>) -> Self {
        StanzaCow::StanzaRef(value)
    }
}
