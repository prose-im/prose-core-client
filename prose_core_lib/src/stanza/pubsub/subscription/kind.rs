use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Kind {
    /// The node MUST NOT send event notifications or payloads to the Entity.
    None,

    /// An entity has requested to subscribe to a node and the request has not yet been approved
    /// by a node owner. The node MUST NOT send event notifications or payloads to the entity while
    /// it is in this state.
    Pending,

    /// An entity has subscribed but its subscription options have not yet been configured. The
    /// node MAY send event notifications or payloads to the entity while it is in this state.
    /// The service MAY timeout unconfigured subscriptions.
    Unconfigured,

    /// An entity is subscribed to a node. The node MUST send all event notifications (and, if
    /// configured, payloads) to the entity while it is in this state (subject to subscriber
    /// configuration and content filtering).
    Subscribed,
}
