// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use url::Url;

use crate::domain::user_info::models::{
    Address as DomainAddress, UserProfile as DomainUserProfile,
};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Address {
    pub locality: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct UserProfile {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub nickname: Option<String>,
    pub org: Option<String>,
    pub role: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
    pub tel: Option<String>,
    pub url: Option<Url>,
    pub address: Option<Address>,
}

impl From<UserProfile> for DomainUserProfile {
    fn from(value: UserProfile) -> Self {
        Self {
            first_name: value.first_name,
            last_name: value.last_name,
            nickname: value.nickname,
            org: value.org,
            role: value.role,
            title: value.title,
            email: value.email,
            tel: value.tel,
            url: value.url,
            address: value.address.map(Into::into),
            photo: None,
        }
    }
}

impl From<DomainUserProfile> for UserProfile {
    fn from(value: DomainUserProfile) -> Self {
        Self {
            first_name: value.first_name,
            last_name: value.last_name,
            nickname: value.nickname,
            org: value.org,
            role: value.role,
            title: value.title,
            email: value.email,
            tel: value.tel,
            url: value.url,
            address: value.address.map(Into::into),
        }
    }
}

impl From<Address> for DomainAddress {
    fn from(value: Address) -> Self {
        Self {
            locality: value.locality,
            country: value.country,
        }
    }
}

impl From<DomainAddress> for Address {
    fn from(value: DomainAddress) -> Self {
        Self {
            locality: value.locality,
            country: value.country,
        }
    }
}
