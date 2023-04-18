use crate::helpers::StanzaCow;
use crate::modules::profile::types::Parameters;
use crate::stanza_base;

pub struct Org<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Org<'a> {
    pub fn new(org: impl AsRef<str>) -> Self {
        Org {
            stanza: Stanza::new("org")
                .add_child(Stanza::new_text_node("text", org))
                .into_inner(),
        }
    }

    pub fn value(&self) -> Option<String> {
        self.child_by_name("text")?.text()
    }

    pub fn parameters(&self) -> Option<Parameters> {
        self.child_by_name("parameters").map(|s| s.into())
    }
}

stanza_base!(Org);
