// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use base64::{engine::general_purpose, Engine as _};
use sha1::{Digest, Sha1};
use xmpp_parsers::hashes::{Algo, Hash};

pub type Namespace = String;

#[derive(Clone, Debug)]
pub struct Capabilities {
    pub node: String,
    pub identity: Identity,
    pub features: Vec<Feature>,
    pub sha1_ver_hash: String,
}

impl Capabilities {
    pub fn new(
        client_name: impl Into<String>,
        client_website: impl Into<String>,
        features: impl IntoIterator<Item = Feature>,
    ) -> Self {
        let identity = Identity {
            category: "client".to_string(),
            kind: "pc".to_string(),
            name: client_name.into(),
        };
        let features: Vec<Feature> = features.into_iter().collect();

        let sha1_ver_hash = Capabilities::sha1_ver_hash(&identity, features.iter());

        Capabilities {
            node: client_website.into(),
            identity,
            features,
            sha1_ver_hash,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Identity {
    pub category: String,
    pub kind: String,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Feature {
    pub var: Namespace,
    pub notify: bool,
}

impl Feature {
    pub fn new(var: impl Into<Namespace>, notify: bool) -> Self {
        Feature {
            var: var.into(),
            notify,
        }
    }
}

impl Display for Feature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.var.to_string(),
            if self.notify { "+notify" } else { "" },
        )
    }
}

impl Capabilities {
    fn sha1_ver_hash<'a>(
        identity: &Identity,
        features: impl Iterator<Item = &'a Feature>,
    ) -> String {
        let mut str = format!(
            "{}/{}/{}/{}<",
            identity.category, identity.kind, "", identity.name
        )
        .into_bytes();

        let mut features: Vec<Vec<u8>> = features
            .into_iter()
            .map(|f| f.to_string().into_bytes())
            .collect();
        features.sort();

        for feat in features {
            str.extend(feat);
            str.extend(b"<");
        }

        let mut hasher = Sha1::new();
        hasher.update(str);
        general_purpose::STANDARD.encode(hasher.finalize())
    }
}

impl From<&Capabilities> for xmpp_parsers::disco::DiscoInfoResult {
    fn from(value: &Capabilities) -> Self {
        xmpp_parsers::disco::DiscoInfoResult {
            node: Some(format!("{}#{}", value.node, value.sha1_ver_hash)),
            identities: vec![(&value.identity).into()],
            features: value.features.iter().map(Into::into).collect(),
            extensions: vec![],
        }
    }
}

impl From<&Capabilities> for xmpp_parsers::caps::Caps {
    fn from(value: &Capabilities) -> Self {
        xmpp_parsers::caps::Caps::new(
            "sha-1",
            Hash {
                algo: Algo::Sha_1,
                hash: value.node.as_bytes().to_vec(),
            },
        )
    }
}

impl From<&Identity> for xmpp_parsers::disco::Identity {
    fn from(value: &Identity) -> Self {
        xmpp_parsers::disco::Identity {
            category: value.category.clone(),
            type_: value.kind.clone(),
            lang: None,
            name: Some(value.name.clone()),
        }
    }
}

impl From<&Feature> for xmpp_parsers::disco::Feature {
    fn from(value: &Feature) -> Self {
        xmpp_parsers::disco::Feature {
            var: value.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use xmpp_parsers::ns;

    use super::*;

    #[test]
    fn test_ver_hash() {
        let caps = Capabilities::new(
            "Exodus 0.9.1",
            "",
            vec![
                Feature::new(ns::MUC, false),
                Feature::new(ns::CAPS, false),
                Feature::new(ns::DISCO_ITEMS, false),
                Feature::new(ns::DISCO_INFO, false),
            ],
        );

        assert_eq!(caps.sha1_ver_hash, "QgayPKawpkPSDYmwT/WM94uAlu0=");
    }
}
