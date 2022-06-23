use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use jid::BareJid;
use libstrophe::Stanza;

#[derive(Debug, PartialEq)]
pub enum PresenceKind {
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

#[derive(Debug, PartialEq)]
pub enum ShowKind {
    /// The entity or resource is temporarily away.
    Away,
    /// The entity or resource is actively interested in chatting.
    Chat,
    /// The entity or resource is busy (dnd = "Do Not Disturb").
    DND,
    /// The entity or resource is away for an extended period (xa = "eXtended Away").
    XA,
}

#[derive(Debug, PartialEq)]
pub struct Presence {
    pub kind: Option<PresenceKind>,
    pub from: Option<BareJid>,
    pub to: Option<BareJid>,
    pub show: Option<ShowKind>,
    pub status: Option<String>,
}

impl TryFrom<&Stanza> for Presence {
    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        Ok(Presence {
            kind: stanza
                .get_attribute("type")
                .map(|s| s.to_string())
                .or_else(|| stanza.get_child_by_name("type").and_then(|n| n.text()))
                .and_then(|s| s.parse::<PresenceKind>().ok()),
            from: stanza
                .get_attribute("from")
                .and_then(|s| BareJid::from_str(s).ok()),
            to: stanza
                .get_attribute("to")
                .and_then(|s| BareJid::from_str(s).ok()),
            show: stanza
                .get_child_by_name("show")
                .and_then(|s| s.text()?.parse::<ShowKind>().ok()),
            status: stanza.get_child_by_name("status").and_then(|n| n.text()),
        })
    }

    type Error = ();
}

impl FromStr for PresenceKind {
    type Err = ();

    fn from_str(input: &str) -> Result<PresenceKind, Self::Err> {
        match input {
            "unavailable" => Ok(PresenceKind::Unavailable),
            "subscribe" => Ok(PresenceKind::Subscribe),
            "subscribed" => Ok(PresenceKind::Subscribed),
            "unsubscribe" => Ok(PresenceKind::Unsubscribe),
            "unsubscribed" => Ok(PresenceKind::Unsubscribed),
            "probe" => Ok(PresenceKind::Probe),
            "error" => Ok(PresenceKind::Error),
            _ => Err(()),
        }
    }
}

impl Display for PresenceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PresenceKind::Unavailable => write!(f, "unavailable"),
            PresenceKind::Subscribe => write!(f, "subscribe"),
            PresenceKind::Subscribed => write!(f, "subscribed"),
            PresenceKind::Unsubscribe => write!(f, "unsubscribe"),
            PresenceKind::Unsubscribed => write!(f, "unsubscribed"),
            PresenceKind::Probe => write!(f, "probe"),
            PresenceKind::Error => write!(f, "error"),
        }
    }
}

impl FromStr for ShowKind {
    type Err = ();

    fn from_str(input: &str) -> Result<ShowKind, Self::Err> {
        match input {
            "away" => Ok(ShowKind::Away),
            "chat" => Ok(ShowKind::Chat),
            "dnd" => Ok(ShowKind::DND),
            "xa" => Ok(ShowKind::XA),
            _ => Err(()),
        }
    }
}

impl Display for ShowKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ShowKind::Away => write!(f, "away"),
            ShowKind::Chat => write!(f, "chat"),
            ShowKind::DND => write!(f, "dnd"),
            ShowKind::XA => write!(f, "xa"),
        }
    }
}

#[cfg(test)]
mod tests {
    use libstrophe::Stanza;

    use super::*;

    #[test]
    fn test_deserialize_empty_presence() {
        let presence = r#"
        <presence/>
        "#;

        let stanza = Stanza::from_str(presence);
        let presence = Presence::try_from(&stanza).unwrap();

        assert_eq!(
            presence,
            Presence {
                kind: None,
                from: None,
                to: None,
                show: None,
                status: None,
            }
        );
    }

    #[test]
    fn test_deserialize_full_presence() {
        let presence = r#"
      <presence from="remi@prose.org" to="marc@prose.org/adium" type="unavailable">
        <show>away</show>
        <status>I'm away</status>
      </presence>
      "#;

        let stanza = Stanza::from_str(presence);
        let presence = Presence::try_from(&stanza).unwrap();

        assert_eq!(
            presence,
            Presence {
                kind: Some(PresenceKind::Unavailable),
                from: Some(BareJid::from_str("remi@prose.org").unwrap()),
                to: Some(BareJid::from_str("marc@prose.org").unwrap()),
                show: Some(ShowKind::Away),
                status: Some("I'm away".to_string()),
            }
        );
    }

    #[test]
    fn test_deserialize_presence_with_type_child() {
        let presence = r#"
      <presence>
        <type>probe</type>
      </presence>
      "#;

        let stanza = Stanza::from_str(presence);
        let presence = Presence::try_from(&stanza).unwrap();

        assert_eq!(
            presence,
            Presence {
                kind: Some(PresenceKind::Probe),
                from: None,
                to: None,
                show: None,
                status: None,
            }
        );
    }
}
