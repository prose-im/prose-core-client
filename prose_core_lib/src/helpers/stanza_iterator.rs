use crate::stanza::Stanza;

pub struct StanzaIterator<'a> {
    children: Box<dyn FnMut() -> Option<libstrophe::StanzaRef<'a>> + 'a>,
}

impl<'a> StanzaIterator<'a> {
    pub fn new(stanza: &'a libstrophe::Stanza) -> Self {
        let mut c = stanza.children();
        let f = move || c.next();

        StanzaIterator {
            children: Box::new(f),
        }
    }
}

impl<'a> Iterator for StanzaIterator<'a> {
    type Item = Stanza<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        (self.children)().map(|s| s.into())
    }
}
