use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::modules::caps::types::feature::Feature;
use crate::modules::caps::types::identity::Identity;
use crate::stanza::Namespace;

pub struct DiscoveryInfo<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> DiscoveryInfo<'a> {
    pub fn new(
        node: impl AsRef<str>,
        identity: Identity,
        features: impl IntoIterator<Item = Feature<'static>>,
    ) -> Self {
        let stanza = Stanza::new("query")
            .set_namespace(Namespace::DiscoInfo)
            .set_attribute("node", node)
            .add_child(identity)
            .add_children(features);

        DiscoveryInfo {
            stanza: stanza.into_inner(),
        }
    }
}

impl<'a> DiscoveryInfo<'a> {}

stanza_base!(DiscoveryInfo);
