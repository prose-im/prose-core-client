use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Show {
    /// The entity or resource is temporarily away.
    Away,
    /// The entity or resource is actively interested in chatting.
    Chat,
    /// The entity or resource is busy (dnd = "Do Not Disturb").
    DND,
    /// The entity or resource is away for an extended period (xa = "eXtended Away").
    XA,
}
