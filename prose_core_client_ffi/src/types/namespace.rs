#[non_exhaustive]
pub struct Namespace;

#[allow(non_upper_case_globals)]
impl Namespace {
    /// XEP-0085: Chat State Notifications
    pub const ChatStates: &'static str = "http://jabber.org/protocol/chatstates";
    /// XEP-0308: Last Message Correction
    pub const LastMessageCorrection: &'static str = "urn:xmpp:message-correct:0";
    /// RFC 6121: XMPP IM
    pub const Roster: &'static str = "jabber:iq:roster";
    /// XEP-0313: Message Archive Management
    pub const MAM2: &'static str = "urn:xmpp:mam:2";
}
