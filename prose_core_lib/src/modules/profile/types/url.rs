use crate::helpers::StanzaCow;
use crate::modules::profile::types::Parameters;
use crate::stanza_base;

pub struct URL<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> URL<'a> {
    pub fn new(url: impl AsRef<str>) -> Self {
        URL {
            stanza: Stanza::new("url")
                .add_child(Stanza::new_text_node("uri", url))
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

stanza_base!(URL);
