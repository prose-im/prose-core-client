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

/// XEP-0045: Multi-User Chat
pub const MUC_ROOMINFO: &str = "http://jabber.org/protocol/muc#roominfo";

/// XEP-0249: Direct MUC Invitations
pub const DIRECT_MUC_INVITATIONS: &str = "jabber:x:conference";

/// XEP-0421: Anonymous unique occupant identifiers for MUCs
pub const OCCUPANT_ID: &str = "urn:xmpp:occupant-id:0";

/// XEP-0066: Out of Band Data
pub const OUT_OF_BAND_DATA: &str = "jabber:x:oob";

/// XEP-0264: Jingle Content Thumbnails
pub const JINGLE_THUMBS: &str = "urn:xmpp:thumbs:1";

/// XEP-0372: References
pub const REFERENCE: &str = "urn:xmpp:reference:0";

/// XEP-0385: Stateless Inline Media Sharing (SIMS)
pub const SIMS: &str = "urn:xmpp:sims:1";

/// XEP-0234: Jingle File Transfer (as used by Movim)
pub const JINGLE_FT_4: &str = "urn:xmpp:jingle:apps:file-transfer:4";

/// Audio Duration in seconds
pub const PROSE_AUDIO_DURATION: &str = "https://prose.org/protocol/audio-duration";

pub const MAM0: &str = "urn:xmpp:mam:0";
pub const MAM1: &str = "urn:xmpp:mam:1";
pub const MAM2: &str = "urn:xmpp:mam:2";
pub const MAM2_EXTENDED: &str = "urn:xmpp:mam:2#extended";
pub const AVATAR_PEP_VCARD_CONVERSION: &str = "urn:xmpp:pep-vcard-conversion:0";

/// XEP-0481: Content Types in Messages
pub const CONTENT: &str = "urn:xmpp:content";

/// XEP-0461: Message Replies
pub const REPLY: &str = "urn:xmpp:reply:0";
