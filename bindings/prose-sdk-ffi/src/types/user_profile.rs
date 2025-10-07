use crate::Url;
use prose_core_client::dtos::{Address as CoreAddress, UserProfile as CoreUserProfile};

#[derive(uniffi::Record)]
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

#[derive(uniffi::Record)]
pub struct Address {
    pub locality: Option<String>,
    pub country: Option<String>,
}

impl From<CoreUserProfile> for UserProfile {
    fn from(user: CoreUserProfile) -> Self {
        Self {
            first_name: user.first_name,
            last_name: user.last_name,
            nickname: user.nickname,
            org: user.org,
            role: user.role,
            title: user.title,
            email: user.email,
            tel: user.tel,
            url: user.url.map(Into::into),
            address: user.address.map(Into::into),
        }
    }
}

impl From<UserProfile> for CoreUserProfile {
    fn from(user: UserProfile) -> Self {
        Self {
            first_name: user.first_name,
            last_name: user.last_name,
            nickname: user.nickname,
            org: user.org,
            role: user.role,
            title: user.title,
            email: user.email,
            tel: user.tel,
            url: user.url.map(Into::into),
            address: user.address.map(Into::into),
        }
    }
}

impl From<CoreAddress> for Address {
    fn from(address: CoreAddress) -> Self {
        Self {
            locality: address.locality,
            country: address.country,
        }
    }
}

impl From<Address> for CoreAddress {
    fn from(address: Address) -> Self {
        Self {
            locality: address.locality,
            country: address.country,
        }
    }
}
