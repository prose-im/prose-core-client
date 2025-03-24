// prose-core-client/prose-xmpp
//
// Copyright:
//   - 2023, Marc Bauer <mb@nesium.com>
//   - 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use minidom::{Element, ElementBuilder};
use xmpp_parsers::iq::IqSetPayload;

use crate::util::ElementExt;
use crate::{ns, ParseError};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VCard4 {
    pub adr: Vec<Adr>,
    pub email: Vec<Email>,
    pub fn_: Vec<Fn_>,
    pub n: Vec<Name>,
    pub impp: Vec<Impp>,
    pub nickname: Vec<Nickname>,
    pub note: Vec<Note>,
    pub org: Vec<Org>,
    pub role: Vec<Role>,
    pub tel: Vec<Tel>,
    pub title: Vec<Title>,
    pub url: Vec<URL>,
    /// Unsupported standard properties, defined in [RFC 6350 - vCard Format Specification, section 3.3](https://datatracker.ietf.org/doc/html/rfc6350#section-3.3) or by IANA (see [vCard Elements](https://www.iana.org/assignments/vcard-elements/vcard-elements.xhtml)).
    pub unknown_: UnsupportedProperties,
    /// Extended properties, as defined in [RFC 6350 - vCard Format Specification, section 6.10](https://datatracker.ietf.org/doc/html/rfc6350#section-6.10).
    pub extensions: UnsupportedProperties,
}

impl VCard4 {
    pub fn new() -> Self {
        VCard4::default()
    }

    pub fn is_empty(&self) -> bool {
        self.adr.is_empty()
            && self.email.is_empty()
            && self.fn_.is_empty()
            && self.n.is_empty()
            && self.impp.is_empty()
            && self.nickname.is_empty()
            && self.note.is_empty()
            && self.org.is_empty()
            && self.role.is_empty()
            && self.tel.is_empty()
            && self.title.is_empty()
            && self.url.is_empty()
            && self.unknown_.is_empty()
            && self.extensions.is_empty()
    }
}

impl IqSetPayload for VCard4 {}

#[derive(Debug, Clone, PartialEq)]
pub struct Fn_ {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Name {
    pub surname: Option<String>,
    pub given: Option<String>,
    pub additional: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Nickname {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Email {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct URL {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Adr {
    pub code: Vec<String>,
    pub country: Vec<String>,
    pub ext: Vec<String>,
    pub locality: Vec<String>,
    pub pobox: Vec<String>,
    pub region: Vec<String>,
    pub street: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Impp {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Org {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnsupportedProperties(Vec<Element>);

impl UnsupportedProperties {
    pub fn get<K: Into<String>>(&self, key: K) -> Vec<&Element> {
        let key = key.into();
        self.0
            .iter()
            .filter_map(|e| {
                if e.name() == &key {
                    Some(e.children())
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
    pub fn push(&mut self, value: Element) {
        self.0.push(value);
    }
    pub fn iter(&self) -> std::slice::Iter<'_, <Self as IntoIterator>::Item> {
        self.0.iter()
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl IntoIterator for UnsupportedProperties {
    type Item = Element;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<V: Into<Element>> FromIterator<V> for UnsupportedProperties {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        Self(Vec::from_iter(iter.into_iter().map(Into::into)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Role {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Title {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tel {
    pub value: String,
}

impl TryFrom<Element> for VCard4 {
    type Error = ParseError;

    fn try_from(root: Element) -> Result<Self, Self::Error> {
        root.expect_is("vcard", ns::VCARD4)?;

        let mut vcard = VCard4::new();

        for child in root.children() {
            match child.name() {
                "fn" => vcard.fn_.push(Fn_ {
                    value: child.text_value()?,
                }),
                "n" => vcard.n.push(Name::try_from(child)?),
                "nickname" => vcard.nickname.push(Nickname {
                    value: child.text_value()?,
                }),
                "email" => vcard.email.push(Email {
                    value: child.text_value()?,
                }),
                "adr" => vcard.adr.push(Adr::try_from(child)?),
                "impp" => vcard.impp.push(Impp {
                    value: child.uri_value()?,
                }),
                "note" => vcard.note.push(Note {
                    value: child.text_value()?,
                }),
                "url" => vcard.url.push(URL {
                    value: child.uri_value()?,
                }),
                "title" => vcard.title.push(Title {
                    value: child.text_value()?,
                }),
                "tel" => vcard.tel.push(Tel {
                    value: child.text_value().or(child
                        .uri_value()
                        .map(|uri| uri.strip_prefix("tel:").unwrap_or(&*uri).to_string()))?,
                }),
                "org" => vcard.org.push(Org {
                    value: child.text_value()?,
                }),
                "role" => vcard.role.push(Role {
                    value: child.text_value()?,
                }),
                key if key.starts_with("x-") => {
                    vcard.extensions.push(child.clone());
                }
                _ => {
                    vcard.unknown_.push(child.clone());
                }
            }
        }

        Ok(vcard)
    }
}

impl From<VCard4> for Element {
    fn from(vcard: VCard4) -> Element {
        Element::builder("vcard", ns::VCARD4)
            .append_all(vcard.adr)
            .append_all_values(vcard.email, "email", "text", |v| v.value)
            .append_all_values(vcard.fn_, "fn", "text", |v| v.value)
            .append_all(vcard.n)
            .append_all_values(vcard.impp, "impp", "uri", |v| v.value)
            .append_all_values(vcard.nickname, "nickname", "text", |v| v.value)
            .append_all_values(vcard.note, "note", "text", |v| v.value)
            .append_all_values(vcard.org, "org", "text", |v| v.value)
            .append_all_values(vcard.role, "role", "text", |v| v.value)
            .append_all_values(vcard.title, "title", "text", |v| v.value)
            .append_all_values(vcard.tel, "tel", "text", |v| v.value)
            .append_all_values(vcard.url, "url", "uri", |v| v.value)
            // See [RFC 6351 - xCard: vCard XML Representation, section 6](https://datatracker.ietf.org/doc/html/rfc6351#section-6).
            .append_all(vcard.unknown_.into_iter())
            // See [RFC 6351 - xCard: vCard XML Representation, section 6](https://datatracker.ietf.org/doc/html/rfc6351#section-6).
            .append_all(vcard.extensions.into_iter())
            .build()
    }
}

impl TryFrom<&Element> for Name {
    type Error = ParseError;

    fn try_from(root: &Element) -> Result<Self, Self::Error> {
        root.expect_is("n", ns::VCARD4)?;

        let mut name = Name::default();

        for child in root.children() {
            match child.name() {
                "surname" => name.surname = Some(child.text()),
                "given" => name.given = Some(child.text()),
                "additional" => name.additional = Some(child.text()),
                _ => (),
            }
        }

        Ok(name)
    }
}

impl From<Name> for Element {
    fn from(value: Name) -> Self {
        Element::builder("n", ns::VCARD4)
            .append_all(
                value
                    .given
                    .map(|given| Element::builder("given", ns::VCARD4).append(given)),
            )
            .append_all(
                value
                    .surname
                    .map(|surname| Element::builder("surname", ns::VCARD4).append(surname)),
            )
            .append_all(
                value.additional.map(|additional| {
                    Element::builder("additional", ns::VCARD4).append(additional)
                }),
            )
            .build()
    }
}

impl TryFrom<&Element> for Adr {
    type Error = ParseError;

    fn try_from(root: &Element) -> Result<Self, Self::Error> {
        let mut adr = Adr::default();

        for child in root.children() {
            match child.name() {
                "code" => adr.code.push(child.text()),
                "country" => adr.country.push(child.text()),
                "ext" => adr.ext.push(child.text()),
                "locality" => adr.locality.push(child.text()),
                "pobox" => adr.pobox.push(child.text()),
                "region" => adr.region.push(child.text()),
                "street" => adr.street.push(child.text()),
                _ => (),
            }
        }

        Ok(adr)
    }
}

impl From<Adr> for Element {
    fn from(adr: Adr) -> Element {
        Element::builder("adr", ns::VCARD4)
            .append_all_strings(adr.code, "code")
            .append_all_strings(adr.country, "country")
            .append_all_strings(adr.ext, "ext")
            .append_all_strings(adr.locality, "locality")
            .append_all_strings(adr.pobox, "pobox")
            .append_all_strings(adr.region, "region")
            .append_all_strings(adr.street, "street")
            .build()
    }
}

trait VCardExt {
    fn text_value(&self) -> Result<String, ParseError>;
    fn uri_value(&self) -> Result<String, ParseError>;
}

trait VCardBuilderExt {
    fn append_all_values<T, F>(
        self,
        vec: Vec<T>,
        name: &str,
        value_name: &str,
        transform: F,
    ) -> Self
    where
        F: Fn(T) -> String;
    fn append_all_strings(self, vec: Vec<String>, name: &str) -> Self;
}

impl VCardExt for Element {
    fn text_value(&self) -> Result<String, ParseError> {
        self.get_child("text", ns::VCARD4)
            .map(|e| e.text())
            .ok_or_else(|| ParseError::Generic {
                msg: format!("Missing element {}.text", self.name()),
            })
    }

    fn uri_value(&self) -> Result<String, ParseError> {
        self.get_child("uri", ns::VCARD4)
            .map(|e| e.text())
            .ok_or_else(|| ParseError::Generic {
                msg: format!("Missing element {}.uri", self.name()),
            })
    }
}

impl VCardBuilderExt for ElementBuilder {
    fn append_all_values<T, F>(
        self,
        vec: Vec<T>,
        name: &str,
        value_name: &str,
        transform: F,
    ) -> Self
    where
        F: Fn(T) -> String,
    {
        if vec.is_empty() {
            return self;
        }

        self.append_all(vec.into_iter().map(|value| {
            Element::builder(name, ns::VCARD4)
                .append(Element::builder(value_name, ns::VCARD4).append(transform(value)))
        }))
    }

    fn append_all_strings(self, vec: Vec<String>, name: &str) -> Self {
        self.append_all(
            vec.into_iter()
                .map(|text| Element::builder(name, ns::VCARD4).append(text).build()),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use crate::test::ElementExt;

    use super::*;

    #[test]
    fn test_deserialize_vcard() -> Result<()> {
        let xml = r#"<vcard xmlns="urn:ietf:params:xml:ns:vcard-4.0">
          <fn><text>Valerian Saliou</text></fn>
          <n>
            <surname>Saliou</surname>
            <given>Valerian</given>
          </n>
          <nickname><text>Valerian</text></nickname>
          <nickname><text>Another nickname</text></nickname>
          <url>
            <uri>https://prose.org/</uri>
          </url>
          <note><text /></note>
          <impp>
            <uri>xmpp:valerian@prose.org</uri>
          </impp>
          <email>
            <parameters><type><text>home</text></type><type><text>work</text></type></parameters>
            <text>valerian@prose.org</text>
          </email>
          <adr>
            <locality>Nantes</locality>
            <country>France, French Republic</country>
          </adr>
        </vcard>
        "#;

        let elem = Element::from_str(xml)?;
        let vcard = VCard4::try_from(elem)?;

        assert_eq!(
            vcard.fn_,
            vec![Fn_ {
                value: "Valerian Saliou".to_string()
            }]
        );
        assert_eq!(
            vcard.n,
            vec![Name {
                surname: Some("Saliou".to_string()),
                given: Some("Valerian".to_string()),
                additional: None,
            }]
        );
        assert_eq!(
            vcard.nickname,
            vec![
                Nickname {
                    value: "Valerian".to_string()
                },
                Nickname {
                    value: "Another nickname".to_string()
                },
            ]
        );
        assert_eq!(
            vcard.url,
            vec![URL {
                value: "https://prose.org/".to_string()
            }]
        );
        assert_eq!(
            vcard.note,
            vec![Note {
                value: "".to_string()
            }]
        );
        assert_eq!(
            vcard.impp,
            vec![Impp {
                value: "xmpp:valerian@prose.org".to_string()
            }]
        );
        assert_eq!(
            vcard.email,
            vec![Email {
                value: "valerian@prose.org".to_string()
            }]
        );
        assert_eq!(
            vcard.adr,
            vec![Adr {
                code: vec![],
                country: vec!["France, French Republic".to_string()],
                ext: vec![],
                locality: vec!["Nantes".to_string()],
                pobox: vec![],
                region: vec![],
                street: vec![],
            }]
        );

        Ok(())
    }

    #[test]
    fn test_remove_tel_prefix() -> Result<()> {
        let xml = r#"<vcard xmlns="urn:ietf:params:xml:ns:vcard-4.0">
            <tel><uri>tel:+1-720-256-6756</uri></tel>
        </vcard>
        "#;

        let elem = Element::from_str(xml)?;
        let vcard = VCard4::try_from(elem)?;

        assert_eq!(
            vcard.tel,
            vec![Tel {
                value: "+1-720-256-6756".to_string()
            }]
        );

        Ok(())
    }

    #[test]
    fn test_serialize_vcard() -> Result<()> {
        let vcard = VCard4 {
            adr: vec![Adr {
                country: vec!["France, French Republic".to_string()],
                locality: vec!["Nantes".to_string()],
                ..Default::default()
            }],
            email: vec![Email {
                value: "valerian@prose.org".to_string(),
            }],
            fn_: vec![Fn_ {
                value: "Valerian Saliou".to_string(),
            }],
            n: vec![Name {
                surname: Some("Saliou".to_string()),
                given: Some("Valerian".to_string()),
                ..Default::default()
            }],
            impp: vec![Impp {
                value: "xmpp:valerian@prose.org".to_string(),
            }],
            nickname: vec![
                Nickname {
                    value: "Valerian".to_string(),
                },
                Nickname {
                    value: "Another nickname".to_string(),
                },
            ],
            url: vec![URL {
                value: "https://prose.org/".to_string(),
            }],
            ..Default::default()
        };

        assert_eq!(VCard4::try_from(Element::from(vcard.clone()))?, vcard);
        Ok(())
    }

    /// We store workspace details in a vCard4, and need to store non-standard properties.
    /// This test ensures extensions are parsed correctly.
    #[test]
    fn test_extensions() -> Result<()> {
        let xml = r#"<vcard xmlns="urn:ietf:params:xml:ns:vcard-4.0">
            <x-color-text><text>#2d8deb</text></x-color-text>
            <x-color-unknown><unknown>#2d8deb</unknown></x-color-unknown>
        </vcard>
        "#;
        let element = Element::from_pretty_printed_xml(xml)?;
        let vcard = VCard4::try_from(element.clone())?;

        assert_eq!(Element::from(vcard.clone()), element);
        // Check that values can be read
        assert_eq!(
            vcard
                .extensions
                .get("x-color-text")
                .first()
                .map(|v| v.text()),
            Some("#2d8deb".to_owned()),
            "{:#?}",
            vcard.extensions.get("x-color-text")
        );
        assert_eq!(
            vcard
                .extensions
                .get("x-color-unknown")
                .first()
                .map(|v| v.text()),
            Some("#2d8deb".to_owned()),
            "{:#?}",
            vcard.extensions.get("x-color-unknown")
        );
        Ok(())
    }

    /// vCard4 standard properties can be extended by IANA, and we shouldn’t drop their value
    /// during a parse/print loop. This test ensures unsupported —yet standard— properties are
    /// kept in such scenario.
    #[test]
    fn test_unknown_properties() -> Result<()> {
        let xml = r#"<vcard xmlns="urn:ietf:params:xml:ns:vcard-4.0">
            <lang><text>en</text></lang>
        </vcard>
        "#;
        let element = Element::from_pretty_printed_xml(xml)?;
        let vcard = VCard4::try_from(element.clone())?;

        assert_eq!(VCard4::try_from(Element::from(vcard.clone()))?, vcard);
        // Check that values can be read
        assert_eq!(
            vcard.unknown_.get("lang").first().map(|v| v.text()),
            Some("en".to_string()),
            "{:#?}",
            vcard.unknown_.get("lang")
        );
        Ok(())
    }
}
