use super::field::Field;
use super::Kind;
use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::stanza::form::field::Kind as FieldKind;
use crate::stanza::Namespace;

// https://xmpp.org/extensions/xep-0004.html

pub struct Form<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Form<'a> {
    pub fn new(kind: Kind) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("x").unwrap();
        stanza.set_ns(Namespace::DataForms.to_string()).unwrap();
        stanza.set_attribute("type", kind.to_string()).unwrap();

        Form {
            stanza: stanza.into(),
        }
    }
}

impl<'a> Form<'a> {
    pub fn set_form_type(self, form_type: impl AsRef<str>) -> Self {
        self.add_field(Field::new("FORM_TYPE", FieldKind::Hidden).add_value(form_type))
    }

    pub fn add_field(self, field: Field) -> Self {
        self.add_child(field)
    }

    pub fn add_field_with_value(
        self,
        var: impl AsRef<str>,
        value: impl AsRef<str>,
        kind: impl Into<Option<FieldKind>>,
    ) -> Self {
        let field = Field::new(var, kind).add_value(value);
        self.add_field(field)
    }
}

stanza_base!(Form);
