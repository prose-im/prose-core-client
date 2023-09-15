// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use prose_xmpp::stanza::muc;
use prose_xmpp::{ns, parse_bool, ParseError};
use std::fmt::Formatter;
use xmpp_parsers::disco;
use xmpp_parsers::disco::DiscoInfoResult;

#[derive(Debug, PartialEq, Clone)]
pub struct RoomSettings {
    pub features: Features,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Features {
    /// Hidden room in Multi-User Chat
    pub is_hidden: bool,
    /// Members-only room in Multi-User Chat
    pub is_members_only: bool,
    /// Moderated room in Multi-User Chat
    pub is_moderated: bool,
    /// Non-anonymous room in Multi-User Chat
    pub is_nonanonymous: bool,
    /// Open room in Multi-User Chat
    pub is_open: bool,
    /// Password-protected room in Multi-User Chat
    pub is_password_protected: bool,
    /// Persistent room in Multi-User Chat
    pub is_persistent: bool,
    /// Public room in Multi-User Chat
    pub is_public: bool,
    /// Semi-anonymous room in Multi-User Chat
    pub is_semianonymous: bool,
    /// Temporary room in Multi-User Chat
    pub is_temporary: bool,
    /// Unmoderated room in Multi-User Chat
    pub is_unmoderated: bool,
    /// Unsecured room in Multi-User Chat
    pub is_unsecured: bool,
    /// Allow members to invite new members
    pub is_member_invites_allowed: bool,
    /// Allow users to invite other users
    pub is_invites_allowed: bool,
    /// XEP-0421: Anonymous unique occupant identifiers for MUCs
    /// https://xmpp.org/extensions/xep-0421.html
    pub supports_occupant_id: bool,
    /// Support for the muc#register FORM_TYPE
    pub supports_registering: bool,
    /// XEP-0077: In-Band Registration
    /// https://xmpp.org/extensions/xep-0077.html#schemas-register
    pub supports_registering_in_band: bool,
    /// https://xmpp.org/extensions/xep-0045.html#registrar-formtype-request
    pub supports_request: bool,
    /// Support for the muc#roomconfig FORM_TYPE
    pub supports_room_config: bool,
    /// Support for the muc#roominfo FORM_TYPE
    pub supports_room_info: bool,
    /// XEP-0410: MUC Self-Ping
    /// https://xmpp.org/extensions/xep-0410.html
    pub supports_self_ping_optimization: bool,
    /// This MUC will reflect the original message 'id' in 'groupchat' messages.
    pub supports_stable_id: bool,
}

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

macro_rules! expect {
    ($msg:expr, $actual:expr, $expected:expr) => {
        FeatureExpectation {
            feature: $msg.to_string(),
            actual_value: $actual,
            expected_value: $expected,
        }
    };
}

impl Features {
    pub fn validate_as_group(&self) -> Result<(), RoomValidationError> {
        use prose_xmpp::stanza::muc::ns::disco_feature as feat;

        Self::validate(
            "Group",
            &[
                expect!(feat::PERSISTENT, self.is_persistent, true),
                expect!(feat::HIDDEN, self.is_hidden, true),
                expect!(feat::MEMBERS_ONLY, self.is_members_only, true),
                expect!(
                    muc::ns::roomconfig::ALLOW_INVITES,
                    self.is_invites_allowed,
                    false
                ),
                expect!(
                    muc::ns::roomconfig::ALLOW_MEMBER_INVITES,
                    self.is_member_invites_allowed,
                    false
                ),
            ],
        )
    }

    pub fn validate_as_private_channel(&self) -> Result<(), RoomValidationError> {
        use prose_xmpp::stanza::muc::ns::disco_feature as feat;

        Self::validate(
            "Private Channel",
            &[
                expect!(feat::PERSISTENT, self.is_persistent, true),
                expect!(feat::HIDDEN, self.is_hidden, true),
                expect!(feat::MEMBERS_ONLY, self.is_members_only, true),
                expect!(
                    muc::ns::roomconfig::ALLOW_INVITES,
                    self.is_invites_allowed,
                    true
                ),
                expect!(
                    muc::ns::roomconfig::ALLOW_MEMBER_INVITES,
                    self.is_member_invites_allowed,
                    true
                ),
            ],
        )
    }

    pub fn validate_as_public_channel(&self) -> Result<(), RoomValidationError> {
        use prose_xmpp::stanza::muc::ns::disco_feature as feat;

        Self::validate(
            "Public Channel",
            &[
                expect!(feat::PERSISTENT, self.is_persistent, true),
                expect!(feat::PUBLIC, self.is_public, true),
                expect!(feat::OPEN, self.is_open, true),
            ],
        )
    }

    pub fn can_act_as_group(&self) -> bool {
        self.validate_as_group().is_ok()
    }

    pub fn can_act_as_private_channel(&self) -> bool {
        self.validate_as_private_channel().is_ok()
    }

    pub fn can_act_as_public_channel(&self) -> bool {
        self.validate_as_public_channel().is_ok()
    }
}

struct FeatureExpectation {
    feature: String,
    actual_value: bool,
    expected_value: bool,
}

impl Features {
    fn validate(
        room_type: &str,
        expectations: &[FeatureExpectation],
    ) -> Result<(), RoomValidationError> {
        let mut failures = vec![];
        for expectation in expectations {
            if expectation.actual_value != expectation.expected_value {
                failures.push(format!(
                    "{} should be {}",
                    expectation.feature,
                    if expectation.expected_value {
                        "true"
                    } else {
                        "false"
                    }
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

impl TryFrom<DiscoInfoResult> for RoomSettings {
    type Error = ParseError;

    fn try_from(value: DiscoInfoResult) -> Result<Self, Self::Error> {
        let features = Features::from(value.features.as_slice());
        let mut result = RoomSettings {
            features,
            name: None,
            description: None,
        };

        for form in &value.extensions {
            if form.form_type.as_deref() != Some(ns::MUC_ROOMINFO) {
                continue;
            }

            for field in &form.fields {
                let Some(var) = &field.var else { continue };

                match var.as_ref() {
                    muc::ns::roominfo::DESCRIPTION => {
                        result.description = field.values.first().cloned()
                    }
                    muc::ns::roomconfig::ROOM_NAME => result.name = field.values.first().cloned(),
                    muc::ns::roomconfig::ALLOW_MEMBER_INVITES => {
                        result.features.is_member_invites_allowed = field
                            .values
                            .first()
                            .map(|value| parse_bool(value))
                            .transpose()?
                            .unwrap_or(false)
                    }
                    muc::ns::roomconfig::ALLOW_INVITES => {
                        result.features.is_invites_allowed = field
                            .values
                            .first()
                            .map(|value| parse_bool(value))
                            .transpose()?
                            .unwrap_or(false)
                    }
                    _ => (),
                }
            }
        }

        Ok(result)
    }
}

impl From<&[disco::Feature]> for Features {
    fn from(features: &[disco::Feature]) -> Self {
        use prose_xmpp::stanza::muc::ns::disco_feature as feat;

        let mut result = Features::default();

        for feature in features {
            match feature.var.as_ref() {
                feat::HIDDEN => result.is_hidden = true,
                feat::MEMBERS_ONLY => result.is_members_only = true,
                feat::MODERATED => result.is_moderated = true,
                feat::NON_ANONYMOUS => result.is_nonanonymous = true,
                feat::OCCUPANT_ID => result.supports_occupant_id = true,
                feat::OPEN => result.is_open = true,
                feat::PASSWORD_PROTECTED => result.is_password_protected = true,
                feat::PERSISTENT => result.is_persistent = true,
                feat::PUBLIC => result.is_public = true,
                feat::REGISTER => result.supports_registering = true,
                feat::REGISTER_IN_BAND => result.supports_registering_in_band = true,
                feat::REQUEST => result.supports_request = true,
                feat::ROOM_CONFIG => result.supports_room_config = true,
                feat::ROOM_INFO => result.supports_room_info = true,
                feat::SELF_PING_OPTIMIZATION => result.supports_self_ping_optimization = true,
                feat::SEMI_ANONYMOUS => result.is_semianonymous = true,
                feat::STABLE_ID => result.supports_stable_id = true,
                feat::TEMPORARY => result.is_temporary = true,
                feat::UNMODERATED => result.is_unmoderated = true,
                feat::UNSECURED => result.is_unsecured = true,
                _ => (),
            }
        }

        result
    }
}
