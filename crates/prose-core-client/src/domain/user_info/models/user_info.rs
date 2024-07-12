// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use crate::domain::shared::models::{Availability, CapabilitiesId};
use crate::domain::shared::utils::ContactNameBuilder;
use crate::domain::user_info::models::{Avatar, JabberClient, UserStatus};
use crate::dtos::{UserBasicInfo, UserId, UserPresenceInfo};

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct UserName {
    /// Name as specified on the roster item representing a contact.
    pub roster: Option<String>,
    /// Name as received via PEP (https://xmpp.org/extensions/xep-0172.html#manage)
    pub nickname: Option<String>,
    /// Name as received via a `nick` element within a presence.
    pub presence: Option<String>,
    /// Name as received/loaded from vCard4/vCard-temp
    pub vcard: Option<ProfileName>,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ProfileName {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub nickname: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct UserInfo {
    pub status: Option<UserStatus>,
    pub availability: Availability,
    pub avatar: Option<Avatar>,
    pub caps: Option<CapabilitiesId>,
    pub client: Option<JabberClient>,
    pub name: UserName,
}

impl UserInfo {
    pub fn display_name(&self) -> ContactNameBuilder {
        self.name.display_name()
    }

    pub fn full_name(&self) -> Option<String> {
        self.name.full_name()
    }

    pub fn into_user_basic_info(self, user_id: UserId) -> UserBasicInfo {
        let name = self.display_name().unwrap_or_username(&user_id);

        UserBasicInfo {
            id: user_id,
            name,
            avatar: self.avatar,
        }
    }

    pub fn into_user_presence_info(self, user_id: UserId) -> UserPresenceInfo {
        let name = self.display_name().unwrap_or_username(&user_id);

        UserPresenceInfo {
            id: user_id,
            name,
            full_name: self.full_name(),
            availability: self.availability,
            avatar: self.avatar,
            status: self.status,
        }
    }
}

impl UserName {
    pub fn display_name(&self) -> ContactNameBuilder {
        ContactNameBuilder::new()
            .or_nickname(self.nickname.as_ref())
            .or_nickname(self.presence.as_ref())
            .or_nickname(
                self.vcard
                    .as_ref()
                    .and_then(|vcard| vcard.nickname.as_ref()),
            )
            .or_firstname_lastname(
                self.vcard
                    .as_ref()
                    .and_then(|vcard| vcard.first_name.as_ref()),
                self.vcard
                    .as_ref()
                    .and_then(|vcard| vcard.last_name.as_ref()),
            )
            .or_nickname(self.roster.as_ref())
    }

    /// Returns the full name only if first and last name are set.
    pub fn full_name(&self) -> Option<String> {
        if self
            .vcard
            .as_ref()
            .map(|vcard| vcard.first_name.is_some() && vcard.last_name.is_some())
            .unwrap_or_default()
        {
            ContactNameBuilder::new()
                .or_firstname_lastname(
                    self.vcard
                        .as_ref()
                        .and_then(|vcard| vcard.first_name.as_ref()),
                    self.vcard
                        .as_ref()
                        .and_then(|vcard| vcard.last_name.as_ref()),
                )
                .build()
        } else {
            None
        }
    }
}

pub trait UserInfoOptExt {
    fn display_name(&self) -> ContactNameBuilder;
    fn into_user_basic_info_or_fallback(self, user_id: UserId) -> UserBasicInfo;
    fn into_user_presence_info_or_fallback(self, user_id: UserId) -> UserPresenceInfo;
}

impl UserInfoOptExt for Option<UserInfo> {
    fn display_name(&self) -> ContactNameBuilder {
        self.as_ref()
            .map(|info| info.display_name())
            .unwrap_or_else(|| ContactNameBuilder::new())
    }

    fn into_user_basic_info_or_fallback(self, user_id: UserId) -> UserBasicInfo {
        let Some(info) = self else {
            let name = self.display_name().unwrap_or_username(&user_id);
            return UserBasicInfo {
                id: user_id,
                name,
                avatar: None,
            };
        };
        info.into_user_basic_info(user_id)
    }

    fn into_user_presence_info_or_fallback(self, user_id: UserId) -> UserPresenceInfo {
        let Some(info) = self else {
            let name = self.display_name().unwrap_or_username(&user_id);
            return UserPresenceInfo {
                id: user_id,
                name,
                full_name: None,
                availability: Availability::Unavailable,
                avatar: None,
                status: None,
            };
        };
        info.into_user_presence_info(user_id)
    }
}
