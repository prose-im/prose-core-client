use anyhow::Result;
use prose_xmpp::stanza::{vcard, VCard4};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct Address {
    pub locality: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct UserProfile {
    pub full_name: Option<String>,
    pub nickname: Option<String>,
    pub org: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
    pub tel: Option<String>,
    pub url: Option<Url>,
    pub address: Option<Address>,
}

trait VecExt {
    type T;

    fn swap_remove_first(&mut self) -> Option<Self::T>;
}

impl<T> VecExt for Vec<T> {
    type T = T;

    fn swap_remove_first(&mut self) -> Option<Self::T> {
        if self.is_empty() {
            return None;
        }
        Some(self.swap_remove(0))
    }
}

impl TryFrom<VCard4> for UserProfile {
    type Error = anyhow::Error;

    fn try_from(mut value: VCard4) -> Result<Self> {
        Ok(UserProfile {
            full_name: value.fn_.swap_remove_first().map(|v| v.value),
            nickname: value.nickname.swap_remove_first().map(|v| v.value),
            org: value.org.swap_remove_first().map(|v| v.value),
            title: value.title.swap_remove_first().map(|v| v.value),
            email: value.email.swap_remove_first().map(|v| v.value),
            tel: value.tel.swap_remove_first().map(|v| v.value),
            url: value
                .url
                .swap_remove_first()
                .and_then(|url| Url::parse(&url.value).ok()),
            address: value.adr.swap_remove_first().map(|mut adr| Address {
                locality: adr.locality.swap_remove_first(),
                country: adr.country.swap_remove_first(),
            }),
        })
    }
}

impl From<UserProfile> for VCard4 {
    fn from(mut value: UserProfile) -> Self {
        let mut vcard = VCard4::new();
        if let Some(full_name) = value.full_name.take() {
            vcard.fn_.push(vcard::Fn_ { value: full_name })
        }
        if let Some(nickname) = value.nickname.take() {
            vcard.nickname.push(vcard::Nickname { value: nickname })
        }
        if let Some(org) = value.org.take() {
            vcard.org.push(vcard::Org { value: org })
        }
        if let Some(title) = value.title.take() {
            vcard.title.push(vcard::Title { value: title })
        }
        if let Some(email) = value.email.take() {
            vcard.email.push(vcard::Email { value: email })
        }
        if let Some(tel) = value.tel.take() {
            vcard.tel.push(vcard::Tel { value: tel })
        }
        if let Some(url) = value.url.take() {
            vcard.url.push(vcard::URL {
                value: url.to_string(),
            })
        }
        if let Some(mut address) = value.address.take() {
            let mut adr = vcard::Adr::new();
            if let Some(locality) = address.locality.take() {
                adr.locality.push(locality)
            }
            if let Some(country) = address.country.take() {
                adr.country.push(country)
            }
            vcard.adr.push(adr)
        }
        vcard
    }
}
