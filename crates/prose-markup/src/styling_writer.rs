// prose-core-client/prose-markup
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use pulldown_cmark_escape::{escape_html, escape_html_body_text, StrWrite};

#[derive(Default)]
struct ListState {
    item_index: Option<u64>,
}

#[derive(Debug, PartialEq)]
enum LinkType {
    Url,
    Reference,
}

struct LinkState {
    r#type: LinkType,
    url: String,
}

pub struct StylingWriter<I, W> {
    /// Iterator supplying events.
    iter: I,

    /// Writer to write to.
    writer: W,

    is_at_start: bool,

    /// Whether the last write wrote a newline.
    end_newline: bool,

    list_state: Vec<ListState>,
    link_state: Option<LinkState>,
    quote_level: usize,
}

impl<'e, I, W> StylingWriter<I, W>
where
    I: Iterator<Item = Event<'e>>,
    W: StrWrite,
{
    pub fn new(iter: I, writer: W) -> Self {
        Self {
            iter,
            writer,
            is_at_start: true,
            end_newline: false,
            list_state: vec![],
            link_state: None,
            quote_level: 0,
        }
    }

    pub fn run(mut self) -> Result<(), W::Error> {
        while let Some(event) = self.iter.next() {
            match event {
                Event::Start(tag) => self.start_tag(tag)?,
                Event::End(tag) => self.end_tag(tag)?,
                Event::Code(text) => {
                    self.write("`")?;
                    self.write(&text)?;
                    self.write("`")?;
                }
                Event::Text(text) | Event::InlineMath(text) | Event::DisplayMath(text) => {
                    self.write_html_body(&text)?;
                }
                Event::Html(html) | Event::InlineHtml(html) => {
                    self.write_html(&html)?;
                }
                Event::SoftBreak | Event::HardBreak => {
                    self.write_newline()?;
                    self.write(
                        &std::iter::repeat(">")
                            .take(self.quote_level)
                            .collect::<String>(),
                    )?;
                    if self.quote_level > 0 {
                        self.write(" ")?;
                    }
                }
                Event::Rule => {
                    self.write("***")?;
                }
                Event::FootnoteReference(_) | Event::TaskListMarker(_) => {}
            }
        }
        Ok(())
    }
}

impl<'e, I, W> StylingWriter<I, W>
where
    I: Iterator<Item = Event<'e>>,
    W: StrWrite,
{
    fn start_tag(&mut self, tag: Tag<'e>) -> Result<(), W::Error> {
        match tag {
            Tag::BlockQuote(_) => {
                self.quote_level += 1;
            }
            Tag::CodeBlock(CodeBlockKind::Fenced(language)) => {
                self.write("```")?;
                self.write(&language)?;
                self.write_newline()?;
            }
            Tag::CodeBlock(CodeBlockKind::Indented) => {
                self.write("```")?;
                self.write_newline()?;
            }
            Tag::Emphasis => {
                self.write("_")?;
            }
            Tag::Item => {
                let idx = self
                    .list_state
                    .last_mut()
                    .and_then(|state| state.item_index.as_mut())
                    .map(|idx| {
                        let current_idx = idx.clone();
                        *idx += 1;
                        current_idx
                    });

                if !self.end_newline {
                    self.write_newline()?;
                }
                self.write(
                    &std::iter::repeat("   ")
                        .take(self.list_state.len() - 1)
                        .collect::<String>(),
                )?;

                if let Some(idx) = idx {
                    self.write(&format!("{idx}. "))?;
                } else {
                    self.write("* ")?;
                }
            }
            Tag::Link { dest_url, .. } => {
                let url = dest_url.to_string();
                let mut link_type = LinkType::Url;

                if url.starts_with("xmpp:") {
                    if let Ok(_) = url[5..].parse::<BareJid>() {
                        link_type = LinkType::Reference;
                    }
                }

                self.link_state = Some(LinkState {
                    r#type: link_type,
                    url,
                })
            }
            Tag::List(idx) => {
                let is_top_level = self.list_state.is_empty();
                self.list_state.push(ListState { item_index: idx });
                if !is_top_level {
                    self.write_newline()?;
                }
            }
            Tag::Strikethrough => {
                self.write("~")?;
            }
            Tag::Strong => {
                self.write("*")?;
            }

            Tag::FootnoteDefinition(_) => {}
            Tag::Heading { .. } => {}
            Tag::HtmlBlock => {}
            Tag::Image { .. } => {}
            Tag::MetadataBlock(_) => {}
            Tag::Paragraph => {
                if !self.end_newline {
                    self.write_newline()?;
                }
                self.write(
                    &std::iter::repeat(">")
                        .take(self.quote_level)
                        .collect::<String>(),
                )?;
                if self.quote_level > 0 {
                    self.write(" ")?;
                }
            }
            Tag::Table(_) => {}
            Tag::TableCell => {}
            Tag::TableHead => {}
            Tag::TableRow => {}
        }

        Ok(())
    }

    fn end_tag(&mut self, tag: TagEnd) -> Result<(), W::Error> {
        match tag {
            TagEnd::BlockQuote => {
                self.quote_level -= 1;

                if !self.end_newline {
                    self.write_newline()?;
                }
            }
            TagEnd::CodeBlock => {
                self.write("```")?;
            }
            TagEnd::Emphasis => {
                self.write("_")?;
            }
            TagEnd::Item => {
                if !self.end_newline {
                    self.write_newline()?;
                }
            }
            TagEnd::Link => {
                if let Some(link_state) = self.link_state.take() {
                    if link_state.r#type == LinkType::Url {
                        self.write(&format!(" ({})", link_state.url))?;
                    }
                }
            }
            TagEnd::List(_) => {
                self.list_state.pop();
                let is_top_level = self.list_state.is_empty();
                if is_top_level {
                    self.write_newline()?;
                }
            }
            TagEnd::Strikethrough => {
                self.write("~")?;
            }
            TagEnd::Strong => {
                self.write("*")?;
            }

            TagEnd::FootnoteDefinition => {}
            TagEnd::Heading(_) => {
                self.write_newline()?;
                self.write_newline()?;
            }
            TagEnd::HtmlBlock => {}
            TagEnd::Image => {}
            TagEnd::MetadataBlock(_) => {}
            TagEnd::Paragraph => {
                if !self.end_newline {
                    self.write_newline()?;
                }
                self.write(
                    &std::iter::repeat(">")
                        .take(self.quote_level)
                        .collect::<String>(),
                )?;
                if self.quote_level > 0 {
                    self.write(" ")?;
                }
                self.write_newline()?;
            }
            TagEnd::Table => {}
            TagEnd::TableCell => {}
            TagEnd::TableHead => {}
            TagEnd::TableRow => {}
        }

        Ok(())
    }

    #[inline]
    fn write(&mut self, s: &str) -> Result<(), W::Error> {
        self.writer.write_str(s)?;
        if !s.is_empty() {
            self.is_at_start = false;
            self.end_newline = s.ends_with('\n');
        }
        Ok(())
    }

    #[inline]
    fn write_html(&mut self, s: &str) -> Result<(), W::Error> {
        escape_html(&mut self.writer, &s)?;
        if !s.is_empty() {
            self.is_at_start = false;
            self.end_newline = s.ends_with('\n');
        }
        Ok(())
    }

    #[inline]
    fn write_html_body(&mut self, s: &str) -> Result<(), W::Error> {
        escape_html_body_text(&mut self.writer, &s)?;
        if !s.is_empty() {
            self.is_at_start = false;
            self.end_newline = s.ends_with('\n');
        }
        Ok(())
    }

    #[inline]
    fn write_newline(&mut self) -> Result<(), W::Error> {
        // Do not start a message with a newline…
        if self.is_at_start {
            return Ok(());
        }

        self.write("\n")?;
        Ok(())
    }
}
