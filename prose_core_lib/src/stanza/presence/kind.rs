use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Kind {
    /// Signals that the entity is no longer available for communication.
    Unavailable,
    /// The sender wishes to subscribe to the recipient's presence.
    Subscribe,
    /// The sender has allowed the recipient to receive their presence.
    Subscribed,
    /// The sender is unsubscribing from another entity's presence.
    Unsubscribe,
    /// The subscription request has been denied or a previously-granted subscription has been cancelled.
    Unsubscribed,
    /// A request for an entity's current presence; SHOULD be generated only by a server on behalf of a user.
    Probe,
    /// An error has occurred regarding processing or delivery of a previously-sent presence stanza.
    Error,
}
