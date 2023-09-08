// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use xmpp_parsers::ns::*;

// See all at: https://xmpp.org/registrar/namespaces.html

/// XEP-0292: vCard4 Over XMPP
pub const VCARD4: &str = "urn:ietf:params:xml:ns:vcard-4.0";

/// XEP-0333: Chat Markers
pub const CHAT_MARKERS: &str = "urn:xmpp:chat-markers:0";

/// XEP-0424: Message Retraction
pub const RETRACT: &str = "urn:xmpp:message-retract:0";

/// XEP-0422: Message Fastening
pub const FASTEN: &str = "urn:xmpp:fasten:0";

/// XEP-0203: Delayed Delivery
pub const DELAY: &str = "urn:xmpp:delay";

/// XEP-0428: Fallback Indication
pub const FALLBACK: &str = "urn:xmpp:fallback:0";

/// XEP-0334: Message Processing Hints
pub const HINTS: &str = "urn:xmpp:hints";

/// XEP-0108: User Activity
pub const USER_ACTIVITY: &str = "http://jabber.org/protocol/activity";

/// XEP-0012: Last Activity
pub const LAST_ACTIVITY: &str = "jabber:iq:last";

/// XEP-0444: Message Reactions
pub const REACTIONS: &str = "urn:xmpp:reactions:0";

/// XEP-0045: Multi-User Chat
pub const MUC_OWNER: &str = "http://jabber.org/protocol/muc#owner";

/// XEP-0045: Multi-User Chat
pub const MUC_ADMIN: &str = "http://jabber.org/protocol/muc#admin";

/// XEP-0045: Multi-User Chat
pub const MUC_ROOMCONFIG: &str = "http://jabber.org/protocol/muc#roomconfig";

/// XEP-0249: Direct MUC Invitations
pub const DIRECT_MUC_INVITATIONS: &str = "jabber:x:conference";
