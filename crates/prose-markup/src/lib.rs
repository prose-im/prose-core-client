// prose-core-client/prose-markup
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use pulldown_cmark::html::push_html;
use pulldown_cmark::{Event, Options, Parser, Tag, TextMergeStream};

use styling_writer::StylingWriter;

mod styling_writer;

#[derive(Debug)]
pub struct MarkdownParser<'input> {
    events: Vec<Event<'input>>,
}

impl<'input> MarkdownParser<'input> {
    pub fn new(s: &'input str) -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        Self {
            events: TextMergeStream::new(Parser::new_ext(s, options).into_iter()).collect(),
        }
    }

    /// Convert Markdown content to XEP-0393: Message Styling
    pub fn convert_to_message_styling(&self) -> String {
        let mut body = String::new();
        let writer = StylingWriter::new(self.events.clone().into_iter(), &mut body);
        writer.run().unwrap();
        body.trim_end().to_string()
    }

    pub fn convert_to_html(&self) -> String {
        let mut html = String::new();
        push_html(&mut html, self.events.clone().into_iter());
        html.trim_end().to_string()
    }

    pub fn collect_mentions(&self) -> Vec<BareJid> {
        self.events
            .iter()
            .filter_map(|event| {
                if let Event::Start(Tag::Link { dest_url, .. }) = event {
                    if dest_url.starts_with("xmpp:") {
                        if let Ok(user_id) = dest_url[5..].parse::<BareJid>() {
                            return Some(user_id);
                        }
                    }
                }
                None
            })
            .collect()
    }
}
