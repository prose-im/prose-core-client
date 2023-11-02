// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

/// https://xmpp.org/extensions/xep-0045.html#registrar-formtype-owner
pub mod roomconfig {
    /// Whether to Allow Occupants to Invite Others
    pub const ALLOW_INVITES: &str = "muc#roomconfig_allowinvites";
    /// Allow members to invite new members
    pub const ALLOW_MEMBER_INVITES: &str =
        "{http://prosody.im/protocol/muc}roomconfig_allowmemberinvites";
    /// Roles that May Send Private Messages
    pub const ALLOW_PM: &str = "muc#roomconfig_allowpm";
    /// Whether to Allow Occupants to Change Subject
    pub const CHANGE_SUBJECT: &str = "muc#roomconfig_changesubject";
    /// Default number of history messages returned by room
    pub const DEFAULT_HISTORY_MESSAGES: &str = "muc#roomconfig_defaulthistorymessages";
    /// Whether to Enable Public Logging of Room Conversations
    pub const ENABLE_LOGGING: &str = "muc#roomconfig_enablelogging";
    /// Roles and Affiliations that May Retrieve Member List
    pub const GET_MEMBER_LIST: &str = "muc#roomconfig_getmemberlist";
    /// Maximum number of history messages returned by room
    pub const HISTORY_LENGTH: &str = "muc#roomconfig_historylength";
    /// Natural Language for Room Discussions
    pub const LANG: &str = "muc#roomconfig_lang";
    /// Maximum Number of History Messages Returned by Room
    pub const MAX_HISTORY_FETCH: &str = "muc#maxhistoryfetch";
    /// Maximum Number of Room Occupants
    pub const MAX_USERS: &str = "muc#roomconfig_maxusers";
    /// Whether to Make Room Members-Only
    pub const MEMBERS_ONLY: &str = "muc#roomconfig_membersonly";
    /// Whether to Make Room Moderated
    pub const MODERATED_ROOM: &str = "muc#roomconfig_moderatedroom";
    /// Whether a Password is Required to Enter
    pub const PASSWORD_PROTECTED_ROOM: &str = "muc#roomconfig_passwordprotectedroom";
    /// Whether to Make Room Persistent
    pub const PERSISTENT_ROOM: &str = "muc#roomconfig_persistentroom";
    /// Roles for which Presence is Broadcasted
    pub const PRESENCE_BROADCAST: &str = "muc#roomconfig_presencebroadcast";
    /// Whether to Allow Public Searching for Room
    pub const PUBLIC_ROOM: &str = "muc#roomconfig_publicroom";
    /// XMPP URI of Associated Publish-Subscribe Node
    pub const PUBSUB: &str = "muc#roomconfig_pubsub";
    /// Full List of Room Admins
    pub const ROOM_ADMINS: &str = "muc#roomconfig_roomadmins";
    /// Short Description of Room
    pub const ROOM_DESC: &str = "muc#roomconfig_roomdesc";
    /// Natural-Language Room Name
    pub const ROOM_NAME: &str = "muc#roomconfig_roomname";
    /// Full List of Room Owners
    pub const ROOM_OWNERS: &str = "muc#roomconfig_roomowners";
    /// The Room Password
    pub const ROOM_SECRET: &str = "muc#roomconfig_roomsecret";
    /// Affiliations that May Discover Real JIDs of Occupants
    pub const WHOIS: &str = "muc#roomconfig_whois";
}

/// https://xmpp.org/extensions/xep-0045.html#registrar-formtype-roominfo
pub mod roominfo {
    /// Contact Addresses (normally, room owner or owners)
    pub const CONTACT_JID: &str = "muc#roominfo_contactjid";
    /// Short Description of Room
    pub const DESCRIPTION: &str = "muc#roominfo_description";
    /// URL for Archived Discussion Logs
    pub const INFO_LOGS: &str = "muc#roominfo_logs";
    /// Natural Language for Room Discussions
    pub const LANG: &str = "muc#roominfo_lang";
    /// An associated LDAP group that defines room membership; this should be an
    /// LDAP Distinguished Name according to an implementation-specific or deployment-specific
    /// definition of a group.
    pub const LDAP_GROUP: &str = "muc#roominfo_ldapgroup";
    /// Maximum Number of History Messages Returned by Room
    pub const MAX_HISTORY_FETCH: &str = "muc#maxhistoryfetch";
    /// Current Number of Occupants in Room
    pub const OCCUPANTS: &str = "muc#roominfo_occupants";
    /// Current Discussion Topic
    pub const SUBJECT: &str = "muc#roominfo_subject";
    /// The room subject can be modified by participants
    pub const SUBJECT_MOD: &str = "muc#roominfo_subjectmod";
}

/// https://xmpp.org/extensions/xep-0045.html#registrar-formtype-register
pub mod register {
    /// Allow this person to register with the room?
    pub const ALLOW: &str = "muc#register_allow";
    /// Email Address
    pub const EMAIL: &str = "muc#register_email";
    /// FAQ Entry
    pub const FAQ_ENTRY: &str = "muc#register_faqentry";
    /// Family Name
    pub const LAST: &str = "muc#register_last";
    /// A Web Page
    pub const REGISTER_URL: &str = "muc#register_url";
    /// Desired Nickname
    pub const ROOM_NICK: &str = "muc#register_roomnick";
}

/// https://xmpp.org/extensions/xep-0045.html#registrar-formtype-request
pub mod request {
    /// User ID
    pub const JID: &str = "muc#jid";
    /// Whether to grant voice
    pub const REQUEST_ALLOW: &str = "muc#request_allow";
    /// Requested role
    pub const ROLE: &str = "muc#role";
    /// Room Nickname
    pub const ROOM_NICK: &str = "muc#roomnick";
}

// https://xmpp.org/extensions/xep-0045.html#registrar-features
pub mod disco_feature {
    /// Hidden room in Multi-User Chat (MUC)
    pub const HIDDEN: &str = "muc_hidden";
    /// Members-only room in Multi-User Chat (MUC)
    pub const MEMBERS_ONLY: &str = "muc_membersonly";
    /// Moderated room in Multi-User Chat (MUC)
    pub const MODERATED: &str = "muc_moderated";
    pub const MUC: &str = "http://jabber.org/protocol/muc";
    /// Non-anonymous room in Multi-User Chat (MUC)
    pub const NON_ANONYMOUS: &str = "muc_nonanonymous";
    /// XEP-0421: Anonymous unique occupant identifiers for MUCs
    /// https://xmpp.org/extensions/xep-0421.html
    pub const OCCUPANT_ID: &str = "urn:xmpp:occupant-id:0";
    /// Open room in Multi-User Chat (MUC)
    pub const OPEN: &str = "muc_open";
    /// Password-protected room in Multi-User Chat (MUC)
    pub const PASSWORD_PROTECTED: &str = "muc_passwordprotected";
    /// Persistent room in Multi-User Chat (MUC)
    pub const PERSISTENT: &str = "muc_persistent";
    /// Public room in Multi-User Chat (MUC)
    pub const PUBLIC: &str = "muc_public";
    /// Support for the muc#register FORM_TYPE
    pub const REGISTER: &str = "http://jabber.org/protocol/muc#register";
    /// XEP-0077: In-Band Registration
    /// https://xmpp.org/extensions/xep-0077.html#schemas-register
    pub const REGISTER_IN_BAND: &str = "jabber:iq:register";
    /// https://xmpp.org/extensions/xep-0045.html#registrar-formtype-request
    pub const REQUEST: &str = "http://jabber.org/protocol/muc#request";
    /// Support for the muc#roomconfig FORM_TYPE
    pub const ROOM_CONFIG: &str = "http://jabber.org/protocol/muc#roomconfig";
    /// Support for the muc#roominfo FORM_TYPE
    pub const ROOM_INFO: &str = "http://jabber.org/protocol/muc#roominfo";
    /// List of MUC rooms (each as a separate item)
    pub const ROOMS: &str = "muc_rooms";
    /// XEP-0410: MUC Self-Ping
    /// https://xmpp.org/extensions/xep-0410.html
    pub const SELF_PING_OPTIMIZATION: &str =
        "http://jabber.org/protocol/muc#self-ping-optimization";
    /// Semi-anonymous room in Multi-User Chat (MUC)
    pub const SEMI_ANONYMOUS: &str = "muc_semianonymous";
    /// This MUC will reflect the original message 'id' in 'groupchat' messages.
    pub const STABLE_ID: &str = "http://jabber.org/protocol/muc#stable_id";
    /// Temporary room in Multi-User Chat (MUC)
    pub const TEMPORARY: &str = "muc_temporary";
    /// Unmoderated room in Multi-User Chat (MUC)
    pub const UNMODERATED: &str = "muc_unmoderated";
    /// Unsecured room in Multi-User Chat (MUC)
    pub const UNSECURED: &str = "muc_unsecured";
}
