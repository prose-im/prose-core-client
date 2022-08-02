#[non_exhaustive]
pub struct Namespace;

// See all at: https://xmpp.org/registrar/namespaces.html

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
    /// XEP-0004: Data Forms
    pub const DataForms: &'static str = "jabber:x:data";
    /// XEP-0297: Stanza Forwarding
    pub const Forward: &'static str = "urn:xmpp:forward:0";
    /// XEP-0203: Delayed Delivery
    pub const Delay: &'static str = "urn:xmpp:delay";
    /// XEP-0359: Unique and Stable Stanza IDs
    pub const StanzaID: &'static str = "urn:xmpp:sid:0";
    /// XEP-0059: Result Set Management
    pub const RSM: &'static str = "http://jabber.org/protocol/rsm";
    /// XEP-0444: Message Reactions
    pub const Reactions: &'static str = "urn:xmpp:reactions:0";
}
