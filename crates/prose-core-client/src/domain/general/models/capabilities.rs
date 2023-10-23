// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub type Namespace = &'static str;

#[derive(Clone, Debug)]
pub struct Capabilities {
    pub node: String,
    pub identity: Identity,
    pub features: Vec<Feature>,
    pub ver_string: String,
}

#[derive(Clone, Debug)]
pub struct Identity {
    pub category: String,
    pub kind: String,
    pub name: String,
    pub lang: String,
}

#[derive(Clone, Debug)]
pub enum Feature {
    Name(Namespace),
    Notify(Namespace),
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
            lang: "en".to_string(),
        };
        let features: Vec<Feature> = features.into_iter().collect();

        let ver_string = Capabilities::ver_string(&identity, features.iter());

        Capabilities {
            node: client_website.into(),
            identity,
            features,
            ver_string,
        }
    }
}

impl Capabilities {
    fn ver_string<'a>(identity: &Identity, features: impl Iterator<Item = &'a Feature>) -> String {
        let mut string = format!(
            "{}/{}/{}/{}<",
            identity.category, identity.kind, identity.lang, identity.name
        );

        let mut features: Vec<String> = features.into_iter().map(|f| f.to_string()).collect();
        features.sort();

        for feat in features {
            string.push_str(&feat);
            string.push_str("<");
        }

        string
    }
}
