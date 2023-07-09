use base64::DecodeError;
use jid::JidParseError;
use strum::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum StanzaParseError {
    #[error("Missing attribute: {attribute}")]
    MissingAttribute { attribute: String },
    #[error("Missing child node: {node}")]
    MissingChildNode { node: String },
    #[error("Parse error: {error}")]
    ParseError { error: String },
    #[error("Jid parse error: {error}")]
    JidParseError { error: JidParseError },
    #[error("Decode error: {error}")]
    DecodeError { error: String },
}

impl StanzaParseError {
    pub fn missing_attribute(attribute: &str) -> Self {
        // TODO: Derive a string for debugging from stanza
        StanzaParseError::MissingAttribute {
            attribute: attribute.to_string(),
        }
    }

    pub fn missing_child_node(node_name: &str) -> Self {
        // TODO: Derive a string for debugging from stanza
        StanzaParseError::MissingChildNode {
            node: node_name.to_string(),
        }
    }
}

impl From<ParseError> for StanzaParseError {
    fn from(error: ParseError) -> Self {
        StanzaParseError::ParseError {
            error: error.to_string(),
        }
    }
}

impl From<JidParseError> for StanzaParseError {
    fn from(error: JidParseError) -> Self {
        StanzaParseError::JidParseError { error }
    }
}

impl From<DecodeError> for StanzaParseError {
    fn from(error: DecodeError) -> Self {
        StanzaParseError::DecodeError {
            error: error.to_string(),
        }
    }
}
