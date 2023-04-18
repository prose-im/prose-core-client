// See all at: https://xmpp.org/registrar/namespaces.html

use strum_macros::{Display, EnumString};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Display, EnumString, Clone)]
pub enum Namespace {
    /// XEP-0085: Chat State Notifications
    #[strum(serialize = "http://jabber.org/protocol/chatstates")]
    ChatStates,

    /// XEP-0308: Last Message Correction
    #[strum(serialize = "urn:xmpp:message-correct:0")]
    LastMessageCorrection,

    /// RFC 6121: XMPP IM
    #[strum(serialize = "jabber:iq:roster")]
    Roster,

    /// XEP-0313: Message Archive Management
    #[strum(serialize = "urn:xmpp:mam:2")]
    MAM2,

    /// XEP-0004: Data Forms
    #[strum(serialize = "jabber:x:data")]
    DataForms,

    /// XEP-0297: Stanza Forwarding
    #[strum(serialize = "urn:xmpp:forward:0")]
    Forward,

    /// XEP-0203: Delayed Delivery
    #[strum(serialize = "urn:xmpp:delay")]
    Delay,

    /// XEP-0359: Unique and Stable Stanza IDs
    #[strum(serialize = "urn:xmpp:sid:0")]
    StanzaID,

    /// XEP-0059: Result Set Management
    #[strum(serialize = "http://jabber.org/protocol/rsm")]
    RSM,

    /// XEP-0444: Message Reactions
    #[strum(serialize = "urn:xmpp:reactions:0")]
    Reactions,

    /// XEP-0422: Message Fastening
    #[strum(serialize = "urn:xmpp:fasten:0")]
    Fasten,

    /// XEP-0424: Message Retraction
    #[strum(serialize = "urn:xmpp:message-retract:0")]
    Retract,

    /// XEP-0428: Fallback Indication
    #[strum(serialize = "urn:xmpp:fallback:0")]
    Fallback,

    /// XEP-0280: Message Carbons
    #[strum(serialize = "urn:xmpp:carbons:2")]
    MessageCarbons,

    /// XEP-0060: Publish-Subscribe
    #[strum(serialize = "http://jabber.org/protocol/pubsub")]
    PubSub,

    /// XEP-0060: Publish-Subscribe
    #[strum(serialize = "http://jabber.org/protocol/pubsub#event")]
    PubSubEvent,

    /// XEP-0084: User Avatars
    #[strum(serialize = "urn:xmpp:avatar:data")]
    AvatarData,

    /// XEP-0084: User Avatars
    #[strum(serialize = "urn:xmpp:avatar:metadata")]
    AvatarMetadata,

    /// XEP-0292: vCard4 Over XMPP
    #[strum(serialize = "urn:ietf:params:xml:ns:vcard-4.0")]
    VCard,

    /// XEP-0199: XMPP Ping
    #[strum(serialize = "urn:xmpp:ping")]
    Ping,

    /// XEP-0184: Message Delivery Receipts
    #[strum(serialize = "urn:xmpp:receipts")]
    Receipts,

    /// XEP-0115: Entity Capabilities
    #[strum(serialize = "http://jabber.org/protocol/caps")]
    Caps,

    /// XEP-0030: Service Discovery
    #[strum(serialize = "http://jabber.org/protocol/disco#info")]
    DiscoInfo,

    /// XEP-0030: Service Discovery
    #[strum(serialize = "http://jabber.org/protocol/disco#items")]
    DiscoItems,

    /// XEP-0045: Multi-User Chat
    #[strum(serialize = "http://jabber.org/protocol/muc")]
    MUC,

    /// XEP-0333: Chat Markers
    #[strum(serialize = "urn:xmpp:chat-markers:0")]
    ChatMarkers,
}
