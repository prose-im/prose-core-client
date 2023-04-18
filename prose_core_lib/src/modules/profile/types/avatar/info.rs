use std::u32;

use crate::helpers::id_string_macro::id_string;
use crate::helpers::StanzaCow;
use crate::stanza_base;

pub struct Info<'a> {
    stanza: StanzaCow<'a>,
}

id_string!(ImageId);

impl<'a> Info<'a> {
    pub fn new(bytes: i64, id: &ImageId, kind: impl AsRef<str>, width: u32, height: u32) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("info").expect("Failed to set name");
        stanza
            .set_attribute("bytes", bytes.to_string())
            .expect("Failed to set attribute");
        stanza
            .set_attribute("id", id)
            .expect("Failed to set attribute");
        stanza
            .set_attribute("type", kind)
            .expect("Failed to set attribute");
        stanza
            .set_attribute("width", width.to_string())
            .expect("Failed to set attribute");
        stanza
            .set_attribute("height", height.to_string())
            .expect("Failed to set attribute");

        Info {
            stanza: stanza.into(),
        }
    }

    pub fn bytes(&self) -> Option<i64> {
        self.attribute("bytes")
            .and_then(|s| i64::from_str_radix(&s, 10).ok())
    }

    pub fn id(&self) -> Option<ImageId> {
        self.attribute("id").map(Into::into)
    }

    pub fn r#type(&self) -> Option<&str> {
        self.attribute("type")
    }

    pub fn width(&self) -> Option<u32> {
        self.attribute("width")
            .and_then(|s| u32::from_str_radix(&s, 10).ok())
    }

    pub fn height(&self) -> Option<u32> {
        self.attribute("height")
            .and_then(|s| u32::from_str_radix(&s, 10).ok())
    }
}

stanza_base!(Info);
