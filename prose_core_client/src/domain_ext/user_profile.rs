use microtype::microtype;
use url::Url;

use prose_core_lib::modules::profile::{Email, Org, Tel, VCard};

microtype! {
    #[derive(Debug)]
    pub prose_core_domain::UserProfile {
        UserProfile
    }
}

impl TryFrom<&VCard<'_>> for UserProfile {
    type Error = anyhow::Error;

    fn try_from(value: &VCard) -> anyhow::Result<Self> {
        Ok(UserProfile(prose_core_domain::UserProfile {
            full_name: value.full_name(),
            nickname: value.nickname(),
            org: value.org().and_then(|p| p.value()),
            title: value.title(),
            email: value.email().and_then(|p| p.value()),
            tel: value.tel().and_then(|p| p.value()),
            url: value.url().and_then(|url| Url::parse(&url).ok()),
            address: value.address().map(|adr| prose_core_domain::Address {
                locality: adr.locality(),
                country: adr.country(),
            }),
        }))
    }
}

impl From<&UserProfile> for VCard<'_> {
    fn from(value: &UserProfile) -> Self {
        let mut vcard = VCard::new();
        if let Some(ref full_name) = value.full_name {
            vcard = vcard.set_full_name(full_name)
        }
        if let Some(ref nickname) = value.nickname {
            vcard = vcard.set_nickname(nickname)
        }
        if let Some(ref org) = value.org {
            vcard = vcard.set_org(Org::new(org))
        }
        if let Some(ref title) = value.title {
            vcard = vcard.set_title(title)
        }
        if let Some(ref email) = value.email {
            vcard = vcard.set_email(Email::new(email))
        }
        if let Some(ref tel) = value.tel {
            vcard = vcard.set_tel(Tel::new(tel))
        }
        if let Some(ref url) = value.url {
            vcard = vcard.set_url(url)
        }
        if let Some(ref address) = value.address {
            let mut adr = prose_core_lib::modules::profile::Address::new();
            if let Some(ref locality) = address.locality {
                adr = adr.set_locality(locality)
            }
            if let Some(ref country) = address.country {
                adr = adr.set_country(country)
            }
            vcard = vcard.set_address(adr)
        }
        vcard
    }
}
