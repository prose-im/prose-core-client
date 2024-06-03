// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::disco;
use xmpp_parsers::disco::DiscoInfoResult;

use prose_xmpp::stanza::muc;
use prose_xmpp::{ns, parse_bool, ParseError};

use crate::domain::shared::models::MamVersion;

#[derive(Debug, PartialEq, Clone)]
pub struct RoomInfo {
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
    /// The supported MAM version
    pub mam_version: Option<MamVersion>,
}

impl TryFrom<DiscoInfoResult> for RoomInfo {
    type Error = ParseError;

    fn try_from(value: DiscoInfoResult) -> Result<Self, Self::Error> {
        let features = Features::from(value.features.as_slice());
        let mut result = RoomInfo {
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
                ns::MAM0 => {
                    result.mam_version = Some(
                        result
                            .mam_version
                            .map_or(MamVersion::Mam0, |v| v.max(MamVersion::Mam0)),
                    )
                }
                ns::MAM1 => {
                    result.mam_version = Some(
                        result
                            .mam_version
                            .map_or(MamVersion::Mam1, |v| v.max(MamVersion::Mam1)),
                    )
                }
                ns::MAM2 => {
                    result.mam_version = Some(
                        result
                            .mam_version
                            .map_or(MamVersion::Mam2, |v| v.max(MamVersion::Mam2)),
                    )
                }
                ns::MAM2_EXTENDED => {
                    result.mam_version =
                        Some(result.mam_version.map_or(MamVersion::Mam2Extended, |v| {
                            v.max(MamVersion::Mam2Extended)
                        }))
                }
                _ => (),
            }
        }

        result
    }
}
