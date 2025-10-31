// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::stanza_error::{DefinedCondition, StanzaError};

pub trait StanzaErrorExt {
    fn to_string(&self) -> String;
}

impl StanzaErrorExt for StanzaError {
    fn to_string(&self) -> String {
        if let Some(text) = self.texts.get("en") {
            return text.clone();
        }
        if let Some((_, text)) = self.texts.first_key_value() {
            return text.clone();
        }
        return self.defined_condition.to_string();
    }
}

impl StanzaErrorExt for DefinedCondition {
    fn to_string(&self) -> String {
        match self {
            DefinedCondition::BadRequest => "Bad Request",
            DefinedCondition::Conflict => "Conflict",
            DefinedCondition::FeatureNotImplemented => "Feature Not Implemented",
            DefinedCondition::Forbidden => "Forbidden",
            DefinedCondition::Gone { .. } => "Gone",
            DefinedCondition::InternalServerError => "Internal Server Error",
            DefinedCondition::ItemNotFound => "Item Not Found",
            DefinedCondition::JidMalformed => "Jid Malformed",
            DefinedCondition::NotAcceptable => "Not Acceptable",
            DefinedCondition::NotAllowed => "Not Allowed",
            DefinedCondition::NotAuthorized => "Not Authorized",
            DefinedCondition::PolicyViolation => "Policy Violation",
            DefinedCondition::RecipientUnavailable => "Recipient Unavailable",
            DefinedCondition::Redirect { .. } => "Redirect",
            DefinedCondition::RegistrationRequired => "Registration Required",
            DefinedCondition::RemoteServerNotFound => "Remote Server Not Found",
            DefinedCondition::RemoteServerTimeout => "Remote Server Timeout",
            DefinedCondition::ResourceConstraint => "Resource Constraint",
            DefinedCondition::ServiceUnavailable => "Service Unavailable",
            DefinedCondition::SubscriptionRequired => "Subscription Required",
            DefinedCondition::UndefinedCondition => "Undefined Condition",
            DefinedCondition::UnexpectedRequest => "Unexpected Request",
        }
        .to_string()
    }
}
