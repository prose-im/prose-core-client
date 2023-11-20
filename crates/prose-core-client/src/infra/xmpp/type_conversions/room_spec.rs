// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::Formatter;

use xmpp_parsers::data_forms::{DataForm, DataFormType};

use prose_xmpp::ns;
use prose_xmpp::stanza::muc;
use prose_xmpp::stanza::muc::ns::disco_feature as feat;
use prose_xmpp::stanza::muc::ns::roomconfig as cfg;

use crate::domain::rooms::models::constants::MAX_PARTICIPANTS_PER_GROUP;
use crate::domain::rooms::models::RoomSpec;
use crate::util::form_config::{FormValue, Value};
use crate::util::{form_config, FormConfig};

use super::room_info::RoomInfo;

#[derive(thiserror::Error, Debug)]
pub struct RoomValidationError {
    room_type: String,
    failures: Vec<String>,
}

impl std::fmt::Display for RoomValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MUC room does not qualify as {}. The following expectations failed:\n{}",
            self.room_type,
            self.failures
                .iter()
                .map(|failure| format!("  - {}", failure))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl RoomSpec {
    pub fn is_satisfied_by(&self, room_info: &RoomInfo) -> bool {
        self.validate_against(room_info).is_ok()
    }
}

impl RoomSpec {
    pub fn validate_against(&self, room_info: &RoomInfo) -> Result<(), RoomValidationError> {
        let room_type: &str;
        let expectations: Vec<(&str, bool, bool)>;
        let features = &room_info.features;

        match self {
            RoomSpec::Group => {
                room_type = "Group";
                expectations = vec![
                    (feat::PERSISTENT, features.is_persistent, true),
                    (feat::HIDDEN, features.is_hidden, true),
                    (feat::MEMBERS_ONLY, features.is_members_only, true),
                    (feat::NON_ANONYMOUS, features.is_nonanonymous, true),
                    (
                        muc::ns::roomconfig::ALLOW_INVITES,
                        features.is_invites_allowed,
                        false,
                    ),
                    (
                        muc::ns::roomconfig::ALLOW_MEMBER_INVITES,
                        features.is_member_invites_allowed,
                        false,
                    ),
                ];
            }
            RoomSpec::PrivateChannel => {
                room_type = "Private Channel";
                expectations = vec![
                    (feat::PERSISTENT, features.is_persistent, true),
                    (feat::HIDDEN, features.is_hidden, true),
                    (feat::MEMBERS_ONLY, features.is_members_only, true),
                    (
                        muc::ns::roomconfig::ALLOW_INVITES,
                        features.is_invites_allowed,
                        true,
                    ),
                    (
                        muc::ns::roomconfig::ALLOW_MEMBER_INVITES,
                        features.is_member_invites_allowed,
                        true,
                    ),
                ];
            }
            RoomSpec::PublicChannel => {
                room_type = "Public Channel";
                expectations = vec![
                    (feat::PERSISTENT, features.is_persistent, true),
                    (feat::PUBLIC, features.is_public, true),
                    (feat::OPEN, features.is_open, true),
                ];
            }
        }

        let mut failures = vec![];
        for expectation in expectations {
            let (feature, actual_value, expected_value) = expectation;

            if actual_value != expected_value {
                failures.push(format!(
                    "{} should be {}",
                    feature,
                    if expected_value { "true" } else { "false" }
                ))
            }
        }
        if failures.is_empty() {
            return Ok(());
        }
        return Err(RoomValidationError {
            room_type: room_type.to_string(),
            failures,
        });
    }
}

impl RoomSpec {
    pub fn populate_form(
        self,
        room_name: &str,
        form: &DataForm,
    ) -> Result<DataForm, form_config::Error> {
        let mut form_values = vec![
            FormValue::optional(cfg::ALLOW_PM, Value::TextSingle("none".to_string())),
            FormValue::optional(cfg::CHANGE_SUBJECT, Value::Boolean(true)),
            FormValue::optional(
                cfg::DEFAULT_HISTORY_MESSAGES,
                Value::TextSingle("0".to_string()),
            ),
            FormValue::optional(cfg::ENABLE_LOGGING, Value::Boolean(false)),
            FormValue::optional(cfg::GET_MEMBER_LIST, Value::Boolean(true)),
            FormValue::optional(cfg::LANG, Value::TextSingle("en".to_string())),
            FormValue::optional(cfg::MAX_HISTORY_FETCH, Value::TextSingle("0".to_string())),
            FormValue::optional(cfg::MODERATED_ROOM, Value::Boolean(false)),
            FormValue::optional(cfg::PASSWORD_PROTECTED_ROOM, Value::Boolean(false)),
            FormValue::optional(cfg::PERSISTENT_ROOM, Value::Boolean(true)),
            FormValue::optional(
                cfg::PRESENCE_BROADCAST,
                Value::ListMulti(vec![
                    "moderator".to_string(),
                    "participant".to_string(),
                    "visitor".to_string(),
                ]),
            ),
            FormValue::optional(cfg::PUBSUB, Value::None),
            FormValue::optional(cfg::ROOM_DESC, Value::None),
            FormValue::optional(cfg::ROOM_OWNERS, Value::None),
            FormValue::optional(cfg::ROOM_SECRET, Value::None),
            FormValue::optional(cfg::WHOIS, Value::ListSingle("anyone".to_string())),
        ];

        let allow_invites: bool;
        let max_users: usize;
        let members_only: bool;

        match self {
            RoomSpec::Group => {
                allow_invites = false;
                max_users = MAX_PARTICIPANTS_PER_GROUP;
                members_only = true;
            }
            RoomSpec::PrivateChannel => {
                allow_invites = true;
                max_users = 100;
                members_only = true;
            }
            RoomSpec::PublicChannel => {
                allow_invites = true;
                max_users = 100;
                members_only = false;
            }
        }

        form_values.extend_from_slice(&[
            FormValue::optional(cfg::ALLOW_INVITES, Value::Boolean(allow_invites)),
            FormValue::optional(cfg::ALLOW_MEMBER_INVITES, Value::Boolean(allow_invites)),
            FormValue::optional(cfg::MAX_USERS, Value::TextSingle(max_users.to_string())),
            FormValue::optional(cfg::MEMBERS_ONLY, Value::Boolean(members_only)),
            FormValue::optional(cfg::PUBLIC_ROOM, Value::Boolean(!members_only)),
            FormValue::optional(cfg::ROOM_NAME, Value::TextSingle(room_name.to_string())),
        ]);

        Ok(DataForm {
            type_: DataFormType::Submit,
            form_type: Some(ns::MUC_ROOMCONFIG.to_string()),
            title: None,
            instructions: None,
            fields: FormConfig::new(form_values).populate_form_fields(&form.fields)?,
        })
    }
}
