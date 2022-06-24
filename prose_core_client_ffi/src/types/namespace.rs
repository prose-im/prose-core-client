use strum_macros::{Display, EnumString};

/// https://xmpp.org/registrar/namespaces.html
#[derive(Debug, Display, EnumString)]
pub enum Namespace {
    /// XEP-0085: Chat State Notifications
    #[strum(serialize = "http://jabber.org/protocol/chatstates")]
    ChatStates,
    /// XEP-0308: Last Message Correction
    #[strum(serialize = "urn:xmpp:message-correct:0")]
    LastMessageCorrection,
}
