use serde::Serialize;
use typeshare::typeshare;

/// A struct representing a bare Jabber ID.
///
/// A bare Jabber ID is composed of 2 components, of which one is optional:
///
///  - A node/name, `node`, which is the optional part before the @.
///  - A domain, `domain`, which is the mandatory part after the @.
///
/// Unlike a `FullJid`, it can’t contain a resource, and should only be used when you are certain
/// there is no case where a resource can be set.  Otherwise, use a `Jid` enum.
#[typeshare]
#[derive(Clone, PartialEq, Eq, Hash, Serialize)]
pub struct BareJid {
    /// The node part of the Jabber ID, if it exists, else None.
    pub node: Option<String>,
    /// The domain of the Jabber ID.
    pub domain: String,
}

/// A struct representing a full Jabber ID.
///
/// A full Jabber ID is composed of 3 components, of which one is optional:
///
///  - A node/name, `node`, which is the optional part before the @.
///  - A domain, `domain`, which is the mandatory part after the @ but before the /.
///  - A resource, `resource`, which is the part after the /.
///
/// Unlike a `BareJid`, it always contains a resource, and should only be used when you are certain
/// there is no case where a resource can be missing.  Otherwise, use a `Jid` enum.
#[typeshare]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FullJid {
    /// The node part of the Jabber ID, if it exists, else None.
    pub node: Option<String>,
    /// The domain of the Jabber ID.
    pub domain: String,
    /// The resource of the Jabber ID.
    pub resource: String,
}

/// An enum representing a Jabber ID. It can be either a `FullJid` or a `BareJid`.
#[typeshare]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "type", content = "content")]
pub enum Jid {
    /// Bare Jid
    Bare(BareJid),

    /// Full Jid
    Full(FullJid),
}
