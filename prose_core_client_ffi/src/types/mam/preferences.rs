// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::error::{Error, StanzaParseError};
use crate::helpers::StanzaExt;
use crate::types::namespace::Namespace;
use jid::BareJid;
use libstrophe::{Stanza, StanzaRef};
use std::collections::HashSet;
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum DefaultBehavior {
    /// All messages are archived by default.
    Always,
    /// Messages are never archived by default.
    Never,
    /// Messages are archived only if the contact's bare JID is in the user's roster.
    Roster,
}

#[derive(Debug, PartialEq)]
pub struct Preferences {
    /// If a JID is in neither the 'always archive' nor the 'never archive' list then whether it
    /// is archived depends on this setting, the default.
    pub default_behavior: DefaultBehavior,

    /// The <prefs/> element MAY contain an <always/> child element. If present, it contains a
    /// list of <jid/> elements, each containing a single JID. The server SHOULD archive any
    /// messages to/from this JID (see 'JID matching').
    pub always_archive: HashSet<BareJid>,

    /// The <prefs/> element MAY contain an <never/> child element. If present, it contains a
    /// list of <jid/> elements, each containing a single JID. The server SHOULD NOT archive any
    /// messages to/from this JID (see 'JID matching').
    pub never_archive: HashSet<BareJid>,
}

impl Preferences {
    pub fn new(
        default_behavior: DefaultBehavior,
        always_archive: HashSet<BareJid>,
        never_archive: HashSet<BareJid>,
    ) -> Self {
        Preferences {
            default_behavior,
            always_archive,
            never_archive,
        }
    }
}

impl TryFrom<&Stanza> for Preferences {
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        Ok(Preferences {
            default_behavior: stanza
                .get_attribute("default")
                .ok_or(StanzaParseError::missing_attribute("default", stanza))
                .and_then(|s| s.parse::<DefaultBehavior>().map_err(Into::into))?,
            always_archive: BareJidHashSet::try_from(&stanza.get_child_by_name("always"))?.0,
            never_archive: BareJidHashSet::try_from(&stanza.get_child_by_name("never"))?.0,
        })
    }
}

/// A FFI-friendly alternative to Preferences since HashSet is not supported by UniFFI yet.
#[derive(Debug, PartialEq)]
pub struct MAMPreferences {
    pub default_behavior: DefaultBehavior,
    pub always_archive: Vec<BareJid>,
    pub never_archive: Vec<BareJid>,
}

impl MAMPreferences {
    pub fn new(
        default_behavior: DefaultBehavior,
        always_archive: Vec<BareJid>,
        never_archive: Vec<BareJid>,
    ) -> Self {
        MAMPreferences {
            default_behavior,
            always_archive,
            never_archive,
        }
    }
}

impl From<Preferences> for MAMPreferences {
    fn from(prefs: Preferences) -> Self {
        let mut prefs = MAMPreferences::new(
            prefs.default_behavior,
            prefs.always_archive.into_iter().collect(),
            prefs.never_archive.into_iter().collect(),
        );
        prefs.always_archive.sort_by_key(|jid| jid.to_string());
        prefs.never_archive.sort_by_key(|jid| jid.to_string());
        prefs
    }
}

impl TryFrom<&MAMPreferences> for Stanza {
    type Error = Error;

    fn try_from(preferences: &MAMPreferences) -> Result<Self, Self::Error> {
        let mut prefs = Stanza::new();
        prefs.set_name("prefs")?;
        prefs.set_ns(Namespace::MAM2)?;
        prefs.set_attribute("default", &preferences.default_behavior.to_string())?;

        let mut always = Stanza::new();
        always.set_name("always")?;
        for jid in &preferences.always_archive {
            let mut jid_node = Stanza::new();
            jid_node.set_name("jid")?;
            jid_node.add_child(Stanza::new_text_node(jid.to_string())?)?;
            always.add_child(jid_node)?;
        }
        prefs.add_child(always)?;

        let mut never = Stanza::new();
        never.set_name("never")?;
        for jid in &preferences.never_archive {
            let mut jid_node = Stanza::new();
            jid_node.set_name("jid")?;
            jid_node.add_child(Stanza::new_text_node(jid.to_string())?)?;
            never.add_child(jid_node)?;
        }
        prefs.add_child(never)?;

        Ok(prefs)
    }
}

struct BareJidHashSet(HashSet<BareJid>);

impl<'a> TryFrom<&Option<StanzaRef<'a>>> for BareJidHashSet {
    type Error = Error;

    fn try_from(value: &Option<StanzaRef>) -> Result<Self, Self::Error> {
        let stanza = match value {
            Some(val) => val,
            None => return Ok(BareJidHashSet(HashSet::new())),
        };

        let mut result = HashSet::<BareJid>::new();
        for child in stanza.children() {
            if child.name() != Some("jid") {
                continue;
            }
            let jid = child
                .text()
                .ok_or(StanzaParseError::missing_text("jid", &*child))
                .and_then(|s| s.parse::<BareJid>().map_err(Into::into))?;
            result.insert(jid);
        }
        Ok(BareJidHashSet(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libstrophe::Stanza;
    use std::str::FromStr;

    #[test]
    fn test_deserialize_preferences_with_missing_archive_configs() {
        let message = r#"<prefs xmlns="urn:xmpp:mam:2" default="roster"/>"#;

        let stanza = Stanza::from_str(message);
        let message = Preferences::try_from(&stanza).unwrap();

        assert_eq!(
            message,
            Preferences::new(DefaultBehavior::Roster, HashSet::new(), HashSet::new())
        );
    }

    #[test]
    fn test_deserialize_preferences_with_empty_archive_configs() {
        let message = r#"<prefs xmlns="urn:xmpp:mam:2" default="roster"><always/><never/></prefs>"#;

        let stanza = Stanza::from_str(message);
        let message = Preferences::try_from(&stanza).unwrap();

        assert_eq!(
            message,
            Preferences::new(DefaultBehavior::Roster, HashSet::new(), HashSet::new())
        );
    }

    #[test]
    fn test_serialize_preferences_with_empty_archive_configs() {
        let prefs = MAMPreferences::new(DefaultBehavior::Roster, vec![], vec![]);
        let stanza = Stanza::try_from(&prefs).unwrap();

        assert_eq!(
            stanza.to_text().unwrap(),
            r#"<prefs default="roster" xmlns="urn:xmpp:mam:2"><always/><never/></prefs>"#
        );
    }

    #[test]
    fn test_deserialize_preferences() {
        let message = r#"<prefs xmlns="urn:xmpp:mam:2" default="always">
            <always><jid>a@prose.org</jid><jid>b@prose.org</jid></always>
            <never><jid>c@prose.org</jid></never>
        </prefs>"#;

        let stanza = Stanza::from_str(message);
        let message = Preferences::try_from(&stanza).unwrap();

        assert_eq!(
            message,
            Preferences::new(
                DefaultBehavior::Always,
                HashSet::from([
                    BareJid::from_str("a@prose.org").unwrap(),
                    BareJid::from_str("b@prose.org").unwrap()
                ]),
                HashSet::from([BareJid::from_str("c@prose.org").unwrap(),])
            )
        );
    }

    #[test]
    fn test_serialize_preferences() {
        let prefs = MAMPreferences::new(
            DefaultBehavior::Always,
            vec![BareJid::from_str("a@prose.org").unwrap()],
            vec![BareJid::from_str("b@prose.org").unwrap()],
        );
        let stanza = Stanza::try_from(&prefs).unwrap();

        assert_eq!(
            stanza.to_text().unwrap(),
            r#"<prefs default="always" xmlns="urn:xmpp:mam:2"><always><jid>a@prose.org</jid></always><never><jid>b@prose.org</jid></never></prefs>"#
        );
    }
}
