// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use anyhow::Result;
use minidom::Element;
use xml::reader::XmlEvent;
use xml::{EmitterConfig, ParserConfig};

pub trait ElementExt {
    fn from_pretty_printed_xml(xml: &str) -> Result<Element>;
    fn to_pretty_printed_xml(&self) -> Result<String>;
}

impl ElementExt for Element {
    fn from_pretty_printed_xml(xml: &str) -> Result<Element> {
        let mut buf = Vec::new();
        to_writer_compact(&mut buf, xml.as_ref())?;
        Ok(Element::from_str(&String::from_utf8(buf)?)?)
    }

    fn to_pretty_printed_xml(&self) -> Result<String> {
        let xml = String::from(self);
        let mut buf = Vec::new();
        to_writer_pretty(&mut buf, xml.as_ref())?;
        Ok(String::from_utf8(buf)?)
    }
}

pub fn to_writer_compact<W>(writer: &mut W, buf: &[u8]) -> std::io::Result<usize>
where
    W: std::io::Write,
{
    let reader = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(true)
        .create_reader(buf);

    let mut writer = EmitterConfig::new()
        .perform_indent(false)
        .normalize_empty_elements(true)
        .autopad_comments(false)
        .write_document_declaration(false)
        .create_writer(writer);

    for event in reader {
        if let Ok(XmlEvent::StartDocument { .. }) = event {
            continue;
        }
        if let Some(event) = event.map_err(to_io)?.as_writer_event() {
            writer.write(event).map_err(to_io)?;
        }
    }
    Ok(buf.len())
}

pub fn to_writer_pretty<W>(writer: &mut W, buf: &[u8]) -> std::io::Result<usize>
where
    W: std::io::Write,
{
    let reader = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(false)
        .create_reader(buf);

    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .normalize_empty_elements(true)
        .autopad_comments(false)
        .write_document_declaration(false)
        .create_writer(writer);

    for event in reader {
        if let Ok(XmlEvent::StartDocument { .. }) = event {
            continue;
        }
        if let Some(event) = event.map_err(to_io)?.as_writer_event() {
            writer.write(event).map_err(to_io)?;
        }
    }
    Ok(buf.len())
}

fn to_io<E>(e: E) -> std::io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    std::io::Error::new(std::io::ErrorKind::Other, e)
}
