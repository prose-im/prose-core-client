use crate::helpers::StanzaCow;
use crate::modules::profile::types::Parameters;
use crate::stanza_base;

pub struct Tel<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Tel<'a> {
    pub fn new(phone: impl AsRef<str>) -> Self {
        Tel {
            stanza: Stanza::new("tel")
                .add_child(Stanza::new_text_node("uri", phone))
                .into_inner(),
        }
    }

    pub fn value(&self) -> Option<String> {
        self.child_by_name("uri")?.text()
    }

    pub fn parameters(&self) -> Option<Parameters> {
        self.child_by_name("parameters").map(|s| s.into())
    }
}

stanza_base!(Tel);
