use crate::helpers::StanzaCow;
use crate::stanza_base;

pub struct Parameters<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Parameters<'a> {
    pub fn new(types: impl IntoIterator<Item = String>) -> Self {
        let stanza = Stanza::new("parameters").add_children(
            types
                .into_iter()
                .map(|t| Stanza::new("type").add_child(Stanza::new_text_node("text", t))),
        );
        Parameters {
            stanza: stanza.into_inner(),
        }
    }

    pub fn types(&self) -> Vec<String> {
        self.children()
            .filter_map(|child| {
                if child.name() != Some("type") {
                    return None;
                }
                return child.child_by_name("text")?.text();
            })
            .collect()
    }
}

stanza_base!(Parameters);
