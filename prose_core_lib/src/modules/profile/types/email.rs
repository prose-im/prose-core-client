use crate::helpers::StanzaCow;
use crate::modules::profile::types::Parameters;
use crate::stanza_base;

pub struct Email<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Email<'a> {
    pub fn new(email: impl AsRef<str>) -> Self {
        Email {
            stanza: Stanza::new("email")
                .add_child(Stanza::new_text_node("text", email))
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

stanza_base!(Email);
