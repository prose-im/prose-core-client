use crate::util::concatenate_names;
use anyhow::Result;
use prose_xmpp::stanza::{vcard, VCard4};
use serde::{Deserialize, Serialize};
pub use url::Url;

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct Address {
    pub locality: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
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

impl UserProfile {
    pub fn full_name(&self) -> Option<String> {
        concatenate_names(&self.first_name, &self.last_name)
    }
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
        let (mut first_name, mut last_name): (Option<String>, Option<String>) = (None, None);

        let name = value.n.swap_remove_first();

        if let Some(
            name @ vcard::Name {
                surname: Some(_), ..
            },
        )
        | Some(name @ vcard::Name { given: Some(_), .. }) = name
        {
            first_name = name.given;
            last_name = name.surname;
        } else if let Some(full_name) = value.fn_.swap_remove_first() {
            let mut split = full_name.value.split(" ");
            first_name = split.next().map(|s| s.to_string());
            last_name = split.next().map(|s| s.to_string());
        }

        Ok(UserProfile {
            first_name,
            last_name,
            nickname: value.nickname.swap_remove_first().map(|v| v.value),
            org: value.org.swap_remove_first().map(|v| v.value),
            role: value.role.swap_remove_first().map(|v| v.value),
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

        if let (Some(first_name), Some(last_name)) = (&value.first_name, &value.last_name) {
            vcard.fn_.push(vcard::Fn_ {
                value: format!("{} {}", first_name, last_name),
            })
        }
        if value.first_name.is_some() || value.last_name.is_some() {
            vcard.n.push(vcard::Name {
                surname: value.last_name.take(),
                given: value.first_name.take(),
                additional: None,
            })
        }
        if let Some(nickname) = value.nickname.take() {
            vcard.nickname.push(vcard::Nickname { value: nickname })
        }
        if let Some(org) = value.org.take() {
            vcard.org.push(vcard::Org { value: org })
        }
        if let Some(role) = value.role.take() {
            vcard.role.push(vcard::Role { value: role })
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
            let mut adr = vcard::Adr::default();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_name() -> Result<()> {
        let mut profile = UserProfile::default();
        assert_eq!(profile.full_name(), None);

        profile.first_name = Some("Jane".to_string());
        assert_eq!(profile.full_name(), Some("Jane".to_string()));

        profile.last_name = Some("Doe".to_string());
        assert_eq!(profile.full_name(), Some("Jane Doe".to_string()));

        profile.first_name = None;
        assert_eq!(profile.full_name(), Some("Doe".to_string()));

        Ok(())
    }

    #[test]
    fn test_convert_to_vcard() -> Result<()> {
        let mut adr = vcard::Adr::default();
        adr.locality = vec!["Berlin".to_string()];
        adr.country = vec!["Germany".to_string()];

        let mut name = vcard::Name::default();
        name.given = Some("John".to_string());
        name.surname = Some("Doe".to_string());

        let mut card = VCard4::default();
        card.adr.push(adr);
        card.n.push(name);
        card.fn_.push(vcard::Fn_ {
            value: "Full Name".to_string(),
        });
        card.nickname.push(vcard::Nickname {
            value: "johndoe".to_string(),
        });
        card.email.push(vcard::Email {
            value: "john.doe@gmail.com".to_string(),
        });
        card.tel.push(vcard::Tel {
            value: "+49123456789".to_string(),
        });
        card.org.push(vcard::Org {
            value: "Acme Inc.".to_string(),
        });
        card.title.push(vcard::Title {
            value: "Ph. D.".to_string(),
        });
        card.role.push(vcard::Role {
            value: "Researcher".to_string(),
        });
        card.url.push(vcard::URL {
            value: "https://www.acme.com/u/john.doe".to_string(),
        });

        let profile = UserProfile::try_from(card.clone())?;
        assert_eq!(
            profile,
            UserProfile {
                first_name: Some("John".to_string()),
                last_name: Some("Doe".to_string()),
                nickname: Some("johndoe".to_string()),
                org: Some("Acme Inc.".to_string()),
                role: Some("Researcher".to_string()),
                title: Some("Ph. D.".to_string()),
                email: Some("john.doe@gmail.com".to_string()),
                tel: Some("+49123456789".to_string()),
                url: Some(Url::parse("https://www.acme.com/u/john.doe")?),
                address: Some(Address {
                    locality: Some("Berlin".to_string()),
                    country: Some("Germany".to_string()),
                }),
            }
        );

        card.fn_ = vec![vcard::Fn_ {
            value: "John Doe".to_string(),
        }];

        assert_eq!(VCard4::from(profile), card);

        Ok(())
    }
}
