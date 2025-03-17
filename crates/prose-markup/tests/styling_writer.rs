// prose-core-client/prose-markup
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use insta::assert_snapshot;
use jid::BareJid;
use pretty_assertions::assert_eq;

use prose_markup::MarkdownParser;

#[test]
fn test_nested_spans() {
    let parser = MarkdownParser::new(include_str!("fixtures/nested_spans.md"));
    assert_snapshot!(parser.convert_to_message_styling());
    assert!(parser.collect_mentions().is_empty());
}

#[test]
fn test_links() {
    let parser = MarkdownParser::new(include_str!("fixtures/links.md"));
    assert_snapshot!(parser.convert_to_message_styling());
    assert_eq!(
        vec!["user@prose.org".parse::<BareJid>().unwrap()],
        parser.collect_mentions()
    );
}

#[test]
fn test_complex() {
    let parser = MarkdownParser::new(include_str!("fixtures/complex.md"));
    assert_snapshot!(parser.convert_to_message_styling());
    assert!(parser.collect_mentions().is_empty());
}

#[test]
fn test_nested_lists() {
    let parser = MarkdownParser::new(include_str!("fixtures/nested_lists.md"));
    assert_snapshot!(parser.convert_to_message_styling());
    assert!(parser.collect_mentions().is_empty());
}

#[test]
fn test_nested_blockquotes() {
    let parser = MarkdownParser::new(include_str!("fixtures/nested_blockquotes.md"));
    assert_snapshot!(parser.convert_to_message_styling());
    assert!(parser.collect_mentions().is_empty());
}

#[test]
fn test_escapes_html_tags() {
    let parser = MarkdownParser::new(include_str!("fixtures/html_tags.md"));
    assert_snapshot!(parser.convert_to_message_styling());
    assert!(parser.collect_mentions().is_empty());
}

#[test]
fn test_plain_text() {
    let parser = MarkdownParser::new("Hello World");
    assert_eq!("Hello World", parser.convert_to_message_styling());
    assert_eq!("<p>Hello World</p>", parser.convert_to_html());
}

#[test]
fn test_fenced_url_is_not_converted_to_anchor() {
    let parser = MarkdownParser::new("`https://www.example.com`");
    assert_eq!(
        "`https://www.example.com`",
        parser.convert_to_message_styling()
    );
    assert_eq!(
        "<p><code>https://www.example.com</code></p>",
        parser.convert_to_html()
    );
}

#[test]
fn test_encodable_entities() {
    let parser = MarkdownParser::new(
        "Already did, it's done by switching the type attribute between text <> password",
    );
    assert_eq!(
        "Already did, it's done by switching the type attribute between text <> password",
        parser.convert_to_message_styling()
    );
    assert_eq!(
        "<p>Already did, it's done by switching the type attribute between text &lt;&gt; password</p>",
        parser.convert_to_html()
    )
}
