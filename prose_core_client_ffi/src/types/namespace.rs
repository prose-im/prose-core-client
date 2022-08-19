// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

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
    /// XEP-0422: Message Fastening
    pub const Fasten: &'static str = "urn:xmpp:fasten:0";
    /// XEP-0424: Message Retraction
    pub const Retract: &'static str = "urn:xmpp:message-retract:0";
    /// XEP-0428: Fallback Indication
    pub const Fallback: &'static str = "urn:xmpp:fallback:0";
    /// XEP-0280: Message Carbons
    pub const MessageCarbons: &'static str = "urn:xmpp:carbons:2";
    /// XEP-0060: Publish-Subscribe
    pub const PubSub: &'static str = "http://jabber.org/protocol/pubsub";
    /// XEP-0060: Publish-Subscribe
    pub const PubSubEvent: &'static str = "http://jabber.org/protocol/pubsub#event";
    /// XEP-0084: User Avatars
    pub const AvatarData: &'static str = "urn:xmpp:avatar:data";
    /// XEP-0084: User Avatars
    pub const AvatarMetadata: &'static str = "urn:xmpp:avatar:metadata";
}
