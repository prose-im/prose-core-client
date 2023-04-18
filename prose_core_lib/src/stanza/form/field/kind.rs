use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Display, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Kind {
    /// The field enables an entity to gather or provide an either-or choice between two options.
    /// The default value is "false".
    Boolean,
    /// The field is intended for data description (e.g., human-readable text such as "section"
    /// headers) rather than data gathering or provision. The <value/> child SHOULD NOT contain
    /// newlines (the \n and \r characters); instead an application SHOULD generate multiple fixed
    /// fields, each with one <value/> child.
    Fixed,
    /// The field is not shown to the form-submitting entity, but instead is returned with the form.
    /// The form-submitting entity SHOULD NOT modify the value of a hidden field, but MAY do so if
    /// such behavior is defined for the "using protocol".
    Hidden,
    /// The field enables an entity to gather or provide multiple Jabber IDs. Each provided JID
    /// SHOULD be unique (as determined by comparison that includes application of the Nodeprep,
    /// Nameprep, and Resourceprep profiles of Stringprep as specified in XMPP Core), and duplicate
    /// JIDs MUST be ignored. *
    JidMulti,
    /// The field enables an entity to gather or provide a single Jabber ID. *
    JidSingle,
    /// The field enables an entity to gather or provide one or more options from among many. A
    /// form-submitting entity chooses one or more items from among the options presented by the
    /// form-processing entity and MUST NOT insert new options. The form-submitting entity MUST
    /// NOT modify the order of items as received from the form-processing entity, since the order
    /// of items MAY be significant.**
    ListMulti,
    /// The field enables an entity to gather or provide one option from among many. A
    /// form-submitting entity chooses one item from among the options presented by the
    /// form-processing entity and MUST NOT insert new options. **
    ListSingle,
    /// The field enables an entity to gather or provide multiple lines of text. ***
    TextMulti,
    /// The field enables an entity to gather or provide a single line or word of text, which
    /// shall be obscured in an interface (e.g., with multiple instances of the asterisk character).
    TextPrivate,
    /// The field enables an entity to gather or provide a single line or word of text, which may
    /// be shown in an interface. This field type is the default and MUST be assumed if a
    /// form-submitting entity receives a field type it does not understand.
    TextSingle,
}

// * Note: Data provided for fields of type "jid-single" or "jid-multi" MUST contain one or more
// valid Jabber IDs, where validity is determined by the addressing rules defined in XMPP Core
// (see the Data Validation section below).
//
// ** Note: The <option/> elements in list-multi and list-single fields MUST be unique, where
// uniqueness is determined by the value of the 'label' attribute and the XML character data of the
// <value/> element (i.e., both must be unique).
//
// *** Note: Data provided for fields of type "text-multi" SHOULD NOT contain any newlines (the \n
// and \r characters). Instead, the application SHOULD split the data into multiple strings (based
// on the newlines inserted by the platform), then specify each string as the XML character data of
// a distinct <value/> element. Similarly, an application that receives multiple <value/> elements
// for a field of type "text-multi" SHOULD merge the XML character data of the value elements into
// one text block for presentation to a user, with each string separated by a newline character as
// appropriate for that platform.

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_serializes_as_kebab_case() {
        assert_eq!(Kind::from_str("text-single").unwrap(), Kind::TextSingle);
        assert_eq!(Kind::TextSingle.to_string(), "text-single");
    }
}
