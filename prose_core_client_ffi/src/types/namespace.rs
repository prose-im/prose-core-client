use strum_macros::{Display, EnumString};

/// https://xmpp.org/registrar/namespaces.html
#[derive(Debug, Display, EnumString)]
pub enum Namespace {
    #[strum(serialize = "http://jabber.org/protocol/chatstates")]
    /// XEP-0085: Chat State Notifications
    ChatStates,
}
