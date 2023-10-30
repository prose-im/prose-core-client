// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Formatter};

use jid::Jid;
use minidom::IntoAttributeValue;
use xmpp_parsers::data_forms::{Field, FieldType};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Boolean(bool),
    JidMulti(Vec<Jid>),
    JidSingle(Jid),
    ListMulti(Vec<String>),
    ListSingle(String),
    TextMulti(Vec<String>),
    TextSingle(String),
    None,
}

#[derive(Debug, Clone)]
pub struct FormValue {
    var: String,
    value: Value,
    required: bool,
}

impl FormValue {
    #[allow(dead_code)]
    pub fn required(var: impl AsRef<str>, value: Value) -> Self {
        Self {
            var: var.as_ref().to_string(),
            value,
            required: true,
        }
    }

    pub fn optional(var: impl AsRef<str>, value: Value) -> Self {
        Self {
            var: var.as_ref().to_string(),
            value,
            required: false,
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    /// The form to be configured is missing fields that are required by the `FormConfig`.
    #[error("Missing required fields: {}", .0.join(", "))]
    MissingRequiredFields(Vec<String>),
    /// The form to be configured contains required fields for which the `FormConfig` doesn't have a value.
    #[error("Missing required value for field {0}")]
    MissingRequiredValue(String),
    /// The provided `Value` doesn't match the type required by the `Field`.
    #[error("Type mismatch in field {var}. Expected type {expected} found type {actual}.")]
    TypeMismatch {
        var: String,
        expected: String,
        actual: String,
    },
    /// The provided `Value` does not exist in the list of available options
    #[error("Invalid value for field {var}. Field does not contain option {value}.")]
    InvalidValue { var: String, value: String },
}

#[derive(Debug, Clone)]
/// https://xmpp.org/extensions/xep-0045.html#registrar-formtype-owner
pub struct FormConfig {
    values: HashMap<String, FormValue>,
}

impl FormConfig {
    pub fn new(values: impl IntoIterator<Item = FormValue>) -> Self {
        FormConfig {
            values: values
                .into_iter()
                .map(|value| (value.var.clone(), value))
                .collect::<HashMap<_, _>>(),
        }
    }

    /// Fills out `fields` and returns the filled out fields.
    pub fn populate_form_fields(&self, fields: &[Field]) -> Result<Vec<Field>, Error> {
        let mut required_fields = self
            .values
            .values()
            .filter_map(|value| {
                if !value.required {
                    return None;
                }
                return Some((value.var.as_str(), ()));
            })
            .collect::<BTreeMap<_, _>>();

        let mut configured_fields = Vec::<Field>::new();

        for field in fields {
            // Continue if the field has no unique identifier…
            let Some(var) = &field.var else {
                continue;
            };

            if field.type_ == FieldType::Fixed {
                continue;
            }

            // Let's see if we have a configured FormValue for this field…
            let Some(value) = self.values.get(var) else {
                // No, but maybe the field already has a (default-)value configured
                // or is not required?
                if !field.values.is_empty() || !field.required {
                    configured_fields.push(field.clone());
                    continue;
                }
                // Nope. So we'll stop right here.
                return Err(Error::MissingRequiredValue(var.clone()));
            };

            // If this is a required FieldValue remove it from the set to mark it as fulfilled.
            if value.required {
                required_fields.remove(var.as_str());
            }

            let mut configured_field = field.clone();
            value.value.apply_to_field(&var, &mut configured_field)?;
            configured_fields.push(configured_field);
        }

        if !required_fields.is_empty() {
            return Err(Error::MissingRequiredFields(
                required_fields
                    .keys()
                    .into_iter()
                    .map(ToString::to_string)
                    .collect(),
            ));
        }

        Ok(configured_fields)
    }
}

impl Value {
    /// https://xmpp.org/extensions/xep-0004.html#table-2
    fn apply_to_field(&self, var: &str, field: &mut Field) -> Result<(), Error> {
        match (self, &field.type_) {
            (Value::None, _) => (),
            (Value::Boolean(value), FieldType::Boolean) => field.values = vec![value.to_string()],
            (Value::JidMulti(values), FieldType::JidMulti) => {
                field.values = values.iter().map(ToString::to_string).collect()
            }
            (Value::JidSingle(value), FieldType::JidSingle) => {
                field.values = vec![value.to_string()]
            }
            (Value::ListMulti(values), FieldType::ListMulti) => {
                for value in values {
                    if !field.options_contains_value(value) {
                        return Err(Error::InvalidValue {
                            var: var.to_string(),
                            value: value.to_string(),
                        });
                    }
                }
                field.values = values.clone()
            }
            (Value::ListSingle(value), FieldType::ListSingle) => {
                if !field.options_contains_value(value) {
                    return Err(Error::InvalidValue {
                        var: var.to_string(),
                        value: value.to_string(),
                    });
                }
                field.values = vec![value.clone()]
            }
            (Value::TextMulti(values), FieldType::TextMulti) => field.values = values.clone(),
            (Value::TextSingle(value), FieldType::TextPrivate) => {
                field.values = vec![value.clone()]
            }
            (Value::TextSingle(value), FieldType::TextSingle) => field.values = vec![value.clone()],
            (_, FieldType::Fixed) => panic!("Unexpected 'fixed' field"),
            // The field is not shown to the form-submitting entity, but instead is returned with
            // the form. The form-submitting entity SHOULD NOT modify the value of a hidden field,
            // but MAY do so if such behavior is defined for the "using protocol".
            (_, FieldType::Hidden) => (),
            (Value::Boolean(_), _)
            | (Value::JidMulti(_), _)
            | (Value::JidSingle(_), _)
            | (Value::ListMulti(_), _)
            | (Value::ListSingle(_), _)
            | (Value::TextMulti(_), _)
            | (Value::TextSingle(_), _) => {
                return Err(Error::TypeMismatch {
                    var: var.to_string(),
                    expected: self.to_string(),
                    actual: field
                        .type_
                        .clone()
                        .into_attribute_value()
                        .unwrap_or("<unknown>".to_string()),
                })
            }
        }
        Ok(())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Value::Boolean(_) => "boolean",
            Value::JidMulti(_) => "jid-multi",
            Value::JidSingle(_) => "jid-single",
            Value::ListMulti(_) => "list-multi",
            Value::ListSingle(_) => "list-single",
            Value::TextMulti(_) => "text-multi",
            Value::TextSingle(_) => "text-single",
            Value::None => "none",
        };
        write!(f, "{}", value)
    }
}

trait FieldExt {
    fn options_contains_value(&self, value: &str) -> bool;
}

impl FieldExt for Field {
    fn options_contains_value(&self, value: &str) -> bool {
        self.options
            .iter()
            .find(|opt| &opt.value == value)
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use xmpp_parsers::data_forms::Option_;

    use prose_xmpp::jid;

    use super::*;

    #[test]
    fn test_bool_field() {
        assert_eq!(
            FormConfig::new([FormValue::optional("field-name", Value::Boolean(true))])
                .populate_form_fields(&[Field::new("field-name", FieldType::Boolean)])
                .unwrap(),
            vec![Field::new("field-name", FieldType::Boolean).with_value("true")]
        )
    }

    #[test]
    fn test_jid_multi() {
        assert_eq!(
            FormConfig::new([FormValue::optional(
                "field-name",
                Value::JidMulti(vec![jid!("a@prose.org"), jid!("b@prose.org")])
            )])
            .populate_form_fields(&[Field::new("field-name", FieldType::JidMulti)])
            .unwrap(),
            vec![Field::new("field-name", FieldType::JidMulti)
                .with_value("a@prose.org")
                .with_value("b@prose.org")]
        )
    }

    #[test]
    fn test_jid_single() {
        assert_eq!(
            FormConfig::new([FormValue::optional(
                "field-name",
                Value::JidSingle(jid!("a@prose.org"))
            )])
            .populate_form_fields(&[Field::new("field-name", FieldType::JidSingle)])
            .unwrap(),
            vec![Field::new("field-name", FieldType::JidSingle).with_value("a@prose.org")]
        )
    }

    #[test]
    fn test_list_multi() {
        let mut field = Field::new("field-name", FieldType::ListMulti);
        field.options = vec![
            Option_ {
                label: None,
                value: "val1".to_string(),
            },
            Option_ {
                label: None,
                value: "val2".to_string(),
            },
        ];

        let mut configured_field = Field::new("field-name", FieldType::ListMulti)
            .with_value("val1")
            .with_value("val2");
        configured_field.options = field.options.clone();

        assert_eq!(
            FormConfig::new([FormValue::optional(
                "field-name",
                Value::ListMulti(vec!["val1".to_string(), "val2".to_string()])
            )])
            .populate_form_fields(&[field])
            .unwrap(),
            vec![configured_field]
        )
    }

    #[test]
    fn test_list_single() {
        let mut field = Field::new("field-name", FieldType::ListSingle);
        field.options = vec![Option_ {
            label: None,
            value: "val1".to_string(),
        }];

        let mut configured_field =
            Field::new("field-name", FieldType::ListSingle).with_value("val1");
        configured_field.options = field.options.clone();

        assert_eq!(
            FormConfig::new([FormValue::optional(
                "field-name",
                Value::ListSingle("val1".to_string())
            )])
            .populate_form_fields(&[field])
            .unwrap(),
            vec![configured_field]
        )
    }

    #[test]
    fn test_list_multi_invalid_value() {
        let mut field = Field::new("field-name", FieldType::ListMulti);
        field.options = vec![Option_ {
            label: None,
            value: "val1".to_string(),
        }];

        assert_eq!(
            FormConfig::new([FormValue::optional(
                "field-name",
                Value::ListMulti(vec!["val2".to_string()])
            )])
            .populate_form_fields(&[field])
            .unwrap_err(),
            Error::InvalidValue {
                var: "field-name".to_string(),
                value: "val2".to_string()
            }
        )
    }

    #[test]
    fn test_list_single_invalid_value() {
        let mut field = Field::new("field-name", FieldType::ListSingle);
        field.options = vec![Option_ {
            label: None,
            value: "val1".to_string(),
        }];

        assert_eq!(
            FormConfig::new([FormValue::optional(
                "field-name",
                Value::ListSingle("val2".to_string())
            )])
            .populate_form_fields(&[field])
            .unwrap_err(),
            Error::InvalidValue {
                var: "field-name".to_string(),
                value: "val2".to_string()
            }
        )
    }

    #[test]
    fn test_text_multi() {
        assert_eq!(
            FormConfig::new([FormValue::optional(
                "field-name",
                Value::TextMulti(vec!["val1".to_string(), "val2".to_string()])
            )])
            .populate_form_fields(&[Field::new("field-name", FieldType::TextMulti)])
            .unwrap(),
            vec![Field::new("field-name", FieldType::TextMulti)
                .with_value("val1")
                .with_value("val2")]
        )
    }

    #[test]
    fn test_text_single() {
        assert_eq!(
            FormConfig::new([FormValue::optional(
                "field-name",
                Value::TextSingle("val1".to_string())
            )])
            .populate_form_fields(&[Field::new("field-name", FieldType::TextSingle)])
            .unwrap(),
            vec![Field::new("field-name", FieldType::TextSingle).with_value("val1")]
        )
    }

    #[test]
    fn test_none() {
        assert_eq!(
            FormConfig::new([FormValue::optional("field-name", Value::None)])
                .populate_form_fields(&[Field::new("field-name", FieldType::TextSingle)])
                .unwrap(),
            vec![Field::new("field-name", FieldType::TextSingle)]
        )
    }

    #[test]
    fn test_text_private() {
        assert_eq!(
            FormConfig::new([FormValue::optional(
                "field-name",
                Value::TextSingle("val1".to_string())
            )])
            .populate_form_fields(&[Field::new("field-name", FieldType::TextPrivate)])
            .unwrap(),
            vec![Field::new("field-name", FieldType::TextPrivate).with_value("val1")]
        )
    }

    #[test]
    fn test_hidden() {
        assert_eq!(
            FormConfig::new([FormValue::optional(
                "f1",
                Value::TextSingle("val1".to_string())
            )])
            .populate_form_fields(&[
                Field::new("f1", FieldType::Hidden),
                Field::new("f2", FieldType::Hidden)
            ])
            .unwrap(),
            vec![
                Field::new("f1", FieldType::Hidden),
                Field::new("f2", FieldType::Hidden)
            ]
        )
    }

    #[test]
    fn test_fixed() {
        let mut field1 = Field::new("f1", FieldType::Fixed).with_value("val1");
        field1.var = None;
        let field2 = Field::new("f1", FieldType::Fixed).with_value("val1");

        assert_eq!(
            FormConfig::new([])
                .populate_form_fields(&[field1, field2])
                .unwrap(),
            vec![]
        )
    }

    #[test]
    fn test_default_value() {
        assert_eq!(
            FormConfig::new([])
                .populate_form_fields(&[
                    Field::new("field-name", FieldType::TextSingle).with_value("val1")
                ])
                .unwrap(),
            vec![Field::new("field-name", FieldType::TextSingle).with_value("val1")]
        )
    }

    #[test]
    fn test_missing_required_fields() {
        assert_eq!(
            FormConfig::new([
                FormValue::required("f1", Value::TextSingle("val1".to_string())),
                FormValue::optional("f2", Value::TextSingle("val1".to_string())),
                FormValue::required("f3", Value::TextSingle("val1".to_string())),
                FormValue::required("f4", Value::TextSingle("val1".to_string()))
            ])
            .populate_form_fields(&[Field::new("f3", FieldType::TextSingle)])
            .unwrap_err(),
            Error::MissingRequiredFields(vec!["f1".to_string(), "f4".to_string()])
        )
    }

    #[test]
    fn test_missing_required_values() {
        let f1 = Field::new("f1", FieldType::TextSingle);
        let mut f2 = Field::new("f2", FieldType::TextSingle);
        f2.required = true;

        assert_eq!(
            FormConfig::new([])
                .populate_form_fields(&[f1, f2])
                .unwrap_err(),
            Error::MissingRequiredValue("f2".to_string())
        )
    }

    #[test]
    fn test_value_mismatch() {
        assert_eq!(
            FormConfig::new([FormValue::optional(
                "field-name",
                Value::TextSingle("val1".to_string())
            ),])
            .populate_form_fields(&[Field::new("field-name", FieldType::JidSingle)])
            .unwrap_err(),
            Error::TypeMismatch {
                var: "field-name".to_string(),
                expected: "text-single".to_string(),
                actual: "jid-single".to_string(),
            }
        )
    }
}
