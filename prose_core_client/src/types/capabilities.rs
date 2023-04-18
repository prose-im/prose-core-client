use std::fmt::{Display, Formatter};

use base64::{engine::general_purpose, Engine as _};
use sha1::{Digest, Sha1};

use prose_core_lib::modules;
use prose_core_lib::stanza;
use prose_core_lib::stanza::Namespace;

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
    pub fn new(var: Namespace, notify: bool) -> Self {
        Feature { var, notify }
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

impl From<&Capabilities> for stanza::presence::Caps<'_> {
    fn from(value: &Capabilities) -> Self {
        stanza::presence::Caps::new("sha-1", &value.node, &value.sha1_ver_hash)
    }
}

impl From<&Capabilities> for modules::caps::DiscoveryInfo<'_> {
    fn from(value: &Capabilities) -> Self {
        modules::caps::DiscoveryInfo::new(
            format!("{}#{}", value.node, value.sha1_ver_hash),
            (&value.identity).into(),
            value.features.iter().map(Into::into),
        )
    }
}

impl From<&Identity> for modules::caps::Identity<'_> {
    fn from(value: &Identity) -> Self {
        modules::caps::Identity::new(&value.category, &value.kind, value.name.as_ref())
    }
}

impl From<&Feature> for modules::caps::Feature<'_> {
    fn from(value: &Feature) -> Self {
        modules::caps::Feature::new(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ver_hash() {
        let caps = Capabilities::new(
            "Exodus 0.9.1",
            "",
            vec![
                Feature::new(Namespace::MUC, false),
                Feature::new(Namespace::Caps, false),
                Feature::new(Namespace::DiscoItems, false),
                Feature::new(Namespace::DiscoInfo, false),
            ],
        );

        assert_eq!(caps.sha1_ver_hash, "QgayPKawpkPSDYmwT/WM94uAlu0=");
    }
}
