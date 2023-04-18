use super::Kind;
use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;

/// A data form of type "form", "submit", or "result" SHOULD contain at least one <field/> element;
/// a data form of type "cancel" SHOULD NOT contain any <field/> elements.
///
/// If the <field/> element type is anything other than "fixed" (see below), it MUST possess a 'var'
/// attribute that uniquely identifies the field in the context of the form (if it is "fixed", it
/// MAY possess a 'var' attribute). The <field/> element MAY possess a 'label' attribute that
/// defines a human-readable name for the field.
//
// The 'type' attribute defines the data "type" of the field data. The following rules apply for
// that attribute:
//
// - For data forms of type "form", each <field/> element SHOULD possess a 'type' attribute. If
//   the 'type' attribute is absent, the default of "text-single" is to be applied.
// - For data forms of type "submit", "result" or "error", the recieving entity can infer the 'type'
//   attribute value from context. Nevertheless, the 'type' attribute MAY be present for clarity.
//   Note that forms of type "error" SHOULD NOT have any <field/> elements.

// If fields are presented in a user interface (e.g., as items in a questionnaire or form result),
// the order of the field elements in the XML SHOULD determine the order of items presented to the
// user.
pub struct Field<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Field<'a> {
    pub fn new<Var: AsRef<str>>(
        var: impl Into<Option<Var>>,
        kind: impl Into<Option<Kind>>,
    ) -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("field").unwrap();

        let var: Option<Var> = var.into();
        if let Some(var) = var {
            stanza.set_attribute("var", var).unwrap();
        }

        let kind: Option<Kind> = kind.into();
        if let Some(kind) = kind {
            stanza.set_attribute("type", kind.to_string()).unwrap();
        }

        Field {
            stanza: stanza.into(),
        }
    }

    /// The XML character data of this element provides a natural-language description of the field,
    /// intended for presentation in a user-agent (e.g., as a "tool-tip", help button, or
    /// xplanatory text provided near the field). The <desc/> element SHOULD NOT contain newlines
    /// (the \n and \r characters), since layout is the responsibility of a user agent, and any
    /// handling of newlines (e.g., presentation in a user interface) is unspecified herein.
    /// (Note: To provide a description of a field, it is RECOMMENDED to use a <desc/> element
    /// rather than a separate <field/> element of type "fixed".)
    pub fn set_description(self, desc: impl AsRef<str>) -> Self {
        self.add_child(Stanza::new_text_node("desc", desc))
    }

    /// This element, which MUST be empty, flags the field as required in order for the form to be
    /// considered valid.
    pub fn set_required(self) -> Self {
        self.add_child(Stanza::new("required"))
    }

    /// The XML character data of this element defines the default value for the field (according
    /// to the form-processing entity) in a data form of type "form", the data provided by a
    /// form-submitting entity in a data form of type "submit", or a data result in a data form of
    /// type "result". In data forms of type "form", if the form-processing entity provides a
    /// default value via the <value/> element, then the form-submitting entity SHOULD NOT attempt
    /// to enforce a different default value (although it MAY do so to respect user preferences or
    /// anticipate expected user input). Fields of type list-multi, jid-multi, text-multi, and
    /// hidden MAY contain more than one <value/> element; all other field types MUST NOT contain
    /// more than one <value/> element.
    pub fn add_value(self, value: impl AsRef<str>) -> Self {
        self.add_child(Stanza::new_text_node("value", value))
    }

    /// One of the options in a field of type "list-single" or "list-multi". The XML character of
    /// the <value/> child defines the option value, and the 'label' attribute defines a
    /// human-readable name for the option. The <option/> element MUST contain one and only one
    /// <value/> child. If the field is not of type "list-single" or "list-multi", it MUST NOT
    /// contain an <option/> element.
    pub fn add_option(self, label: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.add_child(
            Stanza::new("option")
                .set_attribute("label", label)
                .add_child(Stanza::new_text_node("value", value)),
        )
    }
}

stanza_base!(Field);
