// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use url::Url;

use prose_xmpp::stanza::{vcard, vcard4, VCard, VCard4};
use prose_xmpp::RequestError;

use crate::domain::user_info::models::{Address, Image, UserProfile};

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

impl TryFrom<VCard> for UserProfile {
    type Error = RequestError;

    fn try_from(mut value: VCard) -> Result<Self, Self::Error> {
        let (mut first_name, mut last_name): (Option<String>, Option<String>) = (None, None);

        if let Some(
            name @ vcard::Name {
                family: Some(_), ..
            },
        )
        | Some(name @ vcard::Name { given: Some(_), .. }) = value.n
        {
            first_name = name.given;
            last_name = name.family;
        } else if let Some(full_name) = value.fn_ {
            let mut split = full_name.split(" ");
            first_name = split.next().map(|s| s.to_string());
            last_name = split.next().map(|s| s.to_string());
        }

        let nickname = value.nickname;
        let org = value.org.map(|v| v.name);
        let role = value.role;
        let title = value.title;
        let email = value.email.swap_remove_first().map(|v| v.userid);
        let tel = value.tel.swap_remove_first().map(|v| v.number);
        let photo = value.photo.and_then(|img| match img {
            vcard::Image::Binary(media_type, data) => {
                let Some(data) = data.data().ok() else {
                    return None;
                };
                Some(Image::Binary {
                    media_type,
                    data: data.into_owned(),
                })
            }
            vcard::Image::External(url) => url.parse().ok().map(Image::External),
        });

        Ok(UserProfile {
            first_name: trimmed_string(first_name),
            last_name: trimmed_string(last_name),
            nickname: trimmed_string(nickname),
            org: trimmed_string(org),
            role: trimmed_string(role),
            title: trimmed_string(title),
            email: trimmed_string(email),
            tel: trimmed_string(tel),
            url: value.url.and_then(|url| Url::parse(&url).ok()),
            address: value.adr.swap_remove_first().map(|adr| Address {
                locality: trimmed_string(adr.locality),
                country: trimmed_string(adr.country),
            }),
            photo,
        })
    }
}

impl TryFrom<VCard4> for UserProfile {
    type Error = RequestError;

    fn try_from(mut value: VCard4) -> Result<Self, Self::Error> {
        let (mut first_name, mut last_name): (Option<String>, Option<String>) = (None, None);

        let name = value.n.swap_remove_first();

        if let Some(
            name @ vcard4::Name {
                surname: Some(_), ..
            },
        )
        | Some(name @ vcard4::Name { given: Some(_), .. }) = name
        {
            first_name = name.given;
            last_name = name.surname;
        } else if let Some(full_name) = value.fn_.swap_remove_first() {
            let mut split = full_name.value.split(" ");
            first_name = split.next().map(|s| s.to_string());
            last_name = split.next().map(|s| s.to_string());
        }

        let nickname = value.nickname.swap_remove_first().map(|v| v.value);
        let org = value.org.swap_remove_first().map(|v| v.value);
        let role = value.role.swap_remove_first().map(|v| v.value);
        let title = value.title.swap_remove_first().map(|v| v.value);
        let email = value.email.swap_remove_first().map(|v| v.value);
        let tel = value.tel.swap_remove_first().map(|v| v.value);

        Ok(UserProfile {
            first_name: trimmed_string(first_name),
            last_name: trimmed_string(last_name),
            nickname: trimmed_string(nickname),
            org: trimmed_string(org),
            role: trimmed_string(role),
            title: trimmed_string(title),
            email: trimmed_string(email),
            tel: trimmed_string(tel),
            url: value
                .url
                .swap_remove_first()
                .and_then(|url| Url::parse(&url.value).ok()),
            address: value.adr.swap_remove_first().map(|mut adr| Address {
                locality: trimmed_string(adr.locality.swap_remove_first()),
                country: trimmed_string(adr.country.swap_remove_first()),
            }),
            photo: None,
        })
    }
}

impl From<UserProfile> for VCard4 {
    fn from(mut value: UserProfile) -> Self {
        let mut vcard = VCard4::new();

        let first_name = trimmed_string(value.first_name);
        let last_name = trimmed_string(value.last_name);

        if let (Some(first_name), Some(last_name)) = (&first_name, &last_name) {
            vcard.fn_.push(vcard4::Fn_ {
                value: format!("{} {}", first_name, last_name),
            })
        }

        if first_name.is_some() || last_name.is_some() {
            vcard.n.push(vcard4::Name {
                surname: last_name,
                given: first_name,
                additional: None,
            })
        }
        if let Some(nickname) = trimmed_string(value.nickname.take()) {
            vcard.nickname.push(vcard4::Nickname { value: nickname })
        }
        if let Some(org) = trimmed_string(value.org.take()) {
            vcard.org.push(vcard4::Org { value: org })
        }
        if let Some(role) = trimmed_string(value.role.take()) {
            vcard.role.push(vcard4::Role { value: role })
        }
        if let Some(title) = trimmed_string(value.title.take()) {
            vcard.title.push(vcard4::Title { value: title })
        }
        if let Some(email) = trimmed_string(value.email.take()) {
            vcard.email.push(vcard4::Email { value: email })
        }
        if let Some(tel) = trimmed_string(value.tel.take()) {
            vcard.tel.push(vcard4::Tel { value: tel })
        }
        if let Some(url) = value.url.take() {
            vcard.url.push(vcard4::URL {
                value: url.to_string(),
            })
        }
        if let Some(mut address) = value.address.take() {
            let mut adr = vcard4::Adr::default();
            if let Some(locality) = trimmed_string(address.locality.take()) {
                adr.locality.push(locality)
            }
            if let Some(country) = trimmed_string(address.country.take()) {
                adr.country.push(country)
            }
            vcard.adr.push(adr)
        }
        vcard
    }
}

impl From<UserProfile> for VCard {
    fn from(mut value: UserProfile) -> Self {
        let mut vcard = VCard::default();

        let first_name = trimmed_string(value.first_name);
        let last_name = trimmed_string(value.last_name);

        if let (Some(first_name), Some(last_name)) = (&first_name, &last_name) {
            vcard.fn_ = Some(format!("{} {}", first_name, last_name));
        }

        if first_name.is_some() || last_name.is_some() {
            vcard.n = Some(vcard::Name {
                family: last_name,
                given: first_name,
                middle: None,
                prefix: None,
                suffix: None,
            });
        }
        if let Some(nickname) = trimmed_string(value.nickname.take()) {
            vcard.nickname = Some(nickname);
        }
        if let Some(org) = trimmed_string(value.org.take()) {
            vcard.org = Some(vcard::Organization {
                name: org,
                units: vec![],
            });
        }
        if let Some(role) = trimmed_string(value.role.take()) {
            vcard.role = Some(role);
        }
        if let Some(title) = trimmed_string(value.title.take()) {
            vcard.title = Some(title);
        }
        if let Some(email) = trimmed_string(value.email.take()) {
            vcard.email.push(vcard::Email {
                types: vec![],
                userid: email,
            });
        }
        if let Some(tel) = trimmed_string(value.tel.take()) {
            vcard.tel.push(vcard::Telephone {
                types: vec![],
                number: tel,
            })
        }
        if let Some(url) = value.url.take() {
            vcard.url = Some(url.to_string());
        }
        if let Some(mut address) = value.address.take() {
            let mut adr = vcard::Address::default();

            if let Some(locality) = trimmed_string(address.locality.take()) {
                adr.locality = Some(locality);
            }
            if let Some(country) = trimmed_string(address.country.take()) {
                adr.country = Some(country);
            }
            vcard.adr.push(adr)
        }
        vcard
    }
}

fn trimmed_string(string: Option<String>) -> Option<String> {
    let Some(string) = string else {
        return None;
    };

    let trimmed_string = string.trim();
    if trimmed_string.is_empty() {
        return None;
    }

    Some(trimmed_string.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_vcard() -> anyhow::Result<()> {
        let mut adr = vcard4::Adr::default();
        adr.locality = vec!["Berlin".to_string()];
        adr.country = vec!["Germany".to_string()];

        let mut name = vcard4::Name::default();
        name.given = Some("John".to_string());
        name.surname = Some("Doe".to_string());

        let mut card = VCard4::default();
        card.adr.push(adr);
        card.n.push(name);
        card.fn_.push(vcard4::Fn_ {
            value: "Full Name".to_string(),
        });
        card.nickname.push(vcard4::Nickname {
            value: "johndoe".to_string(),
        });
        card.email.push(vcard4::Email {
            value: "john.doe@gmail.com".to_string(),
        });
        card.tel.push(vcard4::Tel {
            value: "+49123456789".to_string(),
        });
        card.org.push(vcard4::Org {
            value: "Acme Inc.".to_string(),
        });
        card.title.push(vcard4::Title {
            value: "Ph. D.".to_string(),
        });
        card.role.push(vcard4::Role {
            value: "Researcher".to_string(),
        });
        card.url.push(vcard4::URL {
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
                photo: None,
            }
        );

        card.fn_ = vec![vcard4::Fn_ {
            value: "John Doe".to_string(),
        }];

        assert_eq!(VCard4::from(profile), card);

        Ok(())
    }
}
