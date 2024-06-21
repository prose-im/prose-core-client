// prose-core-client/prose-xmpp
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::convert::TryFrom;
use std::str::FromStr;

use minidom::Element;

use crate::mods::AvatarData;
use crate::{ns, ElementExt, ParseError};

/// Represents a vCard
#[derive(Debug, Clone, PartialEq, Default)]
pub struct VCard {
    /// Formatted or display name
    pub fn_: Option<String>,
    /// Structured name
    pub n: Option<Name>,
    /// Nicknames as a comma-separated list
    pub nickname: Option<String>,
    /// Photograph (BASE64 encoded binary or URI)
    pub photo: Option<Image>,
    /// Birthday in ISO 8601 format
    pub bday: Option<String>,
    /// Structured addresses
    pub adr: Vec<Address>,
    /// Address labels
    pub label: Vec<Label>,
    /// Telephone numbers
    pub tel: Vec<Telephone>,
    /// Email addresses (default type is INTERNET)
    pub email: Vec<Email>,
    /// Jabber ID in the form of user@host
    pub jabberid: Option<String>,
    /// Mailer (e.g., Mail User Agent Type)
    pub mailer: Option<String>,
    /// Time zone's Standard Time UTC offset (ISO 8601 format)
    pub tz: Option<String>,
    /// Geographical position
    pub geo: Option<Geo>,
    /// Title
    pub title: Option<String>,
    /// Role
    pub role: Option<String>,
    /// Organization logo
    pub logo: Option<Image>,
    /// Administrative agent
    pub agent: Option<Agent>,
    /// Organization name and units
    pub org: Option<Organization>,
    /// Application-specific categories
    pub categories: Vec<String>,
    /// Commentary note
    pub note: Option<String>,
    /// Identifier of product that generated the vCard
    pub prodid: Option<String>,
    /// Last revision date/time (ISO 8601 format)
    pub rev: Option<String>,
    /// Sort string
    pub sort_string: Option<String>,
    /// Formatted name pronunciation
    pub sound: Option<Sound>,
    /// Unique identifier
    pub uid: Option<String>,
    /// Directory URL
    pub url: Option<String>,
    /// Privacy classification
    pub class: Option<Class>,
    /// Authentication credential or encryption key
    pub key: Option<Key>,
    /// Free-form descriptive text
    pub desc: Option<String>,
}

/// Structured name
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Name {
    pub family: Option<String>,
    pub given: Option<String>,
    pub middle: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

/// Photograph (BASE64 encoded binary or URI)
#[derive(Debug, Clone, PartialEq)]
pub enum Image {
    Binary(String, AvatarData), // (type, base64 encoded binary)
    External(String),           // URI
}

/// Structured address
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Address {
    pub types: Vec<AddressType>,
    pub po_box: Option<String>,
    pub ext_add: Option<String>,
    pub street: Option<String>,
    pub locality: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AddressType {
    Home,
    Work,
    Postal,
    Parcel,
    Dom,
    Intl,
    Pref,
}

/// Address label
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Label {
    pub types: Vec<AddressType>,
    /// Individual lines
    pub lines: Vec<String>,
}

/// Telephone number
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Telephone {
    pub types: Vec<TelType>,
    pub number: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TelType {
    Home,
    Work,
    Voice,
    Fax,
    Pager,
    Msg,
    Cell,
    Video,
    Bbs,
    Modem,
    Isdn,
    Pcs,
    Pref,
}

/// Email address
#[derive(Debug, Clone, PartialEq)]
pub struct Email {
    pub types: Vec<EmailType>,
    pub userid: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EmailType {
    Home,
    Work,
    Internet,
    Pref,
    X400,
}

/// Geographical position (decimal degrees, six decimal places)
#[derive(Debug, Clone, PartialEq)]
pub struct Geo {
    pub lat: f64,
    pub lon: f64,
}

/// Administrative agent
#[derive(Debug, Clone, PartialEq)]
pub enum Agent {
    VCard(Box<VCard>),
    External(String), // URI
}

/// Organization name and units
#[derive(Debug, Clone, PartialEq)]
pub struct Organization {
    pub name: String,
    pub units: Vec<String>,
}

/// Formatted name pronunciation
#[derive(Debug, Clone, PartialEq)]
pub enum Sound {
    Phonetic(String),
    Binary(String),   // base64 encoded binary
    External(String), // URI
}

/// Privacy classification
#[derive(Debug, Clone, PartialEq)]
pub enum Class {
    Public,
    Private,
    Confidential,
}

/// Authentication credential or encryption key
#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    pub type_: Option<String>,
    pub cred: String,
}

impl TryFrom<&Element> for VCard {
    type Error = ParseError;

    fn try_from(root: &Element) -> Result<Self, Self::Error> {
        root.expect_is("vCard", ns::VCARD)?;

        let mut vcard = VCard::default();

        for child in root.children() {
            match child.name() {
                "FN" => vcard.fn_ = child.non_empty_text(),
                "N" => vcard.n = Some(Name::try_from(child)?),
                "NICKNAME" => vcard.nickname = child.non_empty_text(),
                "PHOTO" => vcard.photo = Some(Image::try_from(child)?),
                "BDAY" => vcard.bday = child.non_empty_text(),
                "ADR" => vcard.adr.push(Address::try_from(child)?),
                "LABEL" => vcard.label.push(Label::try_from(child)?),
                "TEL" => {
                    let tel = Telephone::try_from(child)?;
                    if !tel.number.is_empty() {
                        vcard.tel.push(tel);
                    }
                }
                "EMAIL" => {
                    let email = Email::try_from(child)?;
                    if !email.userid.is_empty() {
                        vcard.email.push(email);
                    }
                }
                "JABBERID" => vcard.jabberid = child.non_empty_text(),
                "MAILER" => vcard.mailer = child.non_empty_text(),
                "TZ" => vcard.tz = child.non_empty_text(),
                "GEO" => vcard.geo = Some(Geo::try_from(child)?),
                "TITLE" => vcard.title = child.non_empty_text(),
                "ROLE" => vcard.role = child.non_empty_text(),
                "LOGO" => vcard.logo = Some(Image::try_from(child)?),
                "AGENT" => vcard.agent = Some(Agent::try_from(child)?),
                "ORG" => vcard.org = Some(Organization::try_from(child)?),
                "CATEGORIES" => {
                    vcard.categories = child.text().split(',').map(String::from).collect()
                }
                "NOTE" => vcard.note = child.non_empty_text(),
                "PRODID" => vcard.prodid = child.non_empty_text(),
                "REV" => vcard.rev = child.non_empty_text(),
                "SORT-STRING" => vcard.sort_string = child.non_empty_text(),
                "SOUND" => vcard.sound = Some(Sound::try_from(child)?),
                "UID" => vcard.uid = child.non_empty_text(),
                "URL" => vcard.url = child.non_empty_text(),
                "KEY" => vcard.key = Some(Key::try_from(child)?),
                "DESC" => vcard.desc = child.non_empty_text(),
                _ => {
                    if let Ok(class) = child.name().parse() {
                        vcard.class = Some(class);
                    }
                }
            }
        }

        Ok(vcard)
    }
}

impl TryFrom<&Element> for Name {
    type Error = ParseError;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        element.expect_is("N", ns::VCARD)?;

        let mut name = Name::default();

        for child in element.children() {
            match child.name() {
                "FAMILY" => name.family = child.non_empty_text(),
                "GIVEN" => name.given = child.non_empty_text(),
                "MIDDLE" => name.middle = child.non_empty_text(),
                "PREFIX" => name.prefix = child.non_empty_text(),
                "SUFFIX" => name.suffix = child.non_empty_text(),
                _ => {}
            }
        }

        Ok(name)
    }
}

impl TryFrom<&Element> for Image {
    type Error = ParseError;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        let mut binval = None;
        let mut extval = None;
        let mut r#type = None;

        for child in element.children() {
            match child.name() {
                "BINVAL" => binval = Some(child.text()),
                "EXTVAL" => extval = Some(child.text()),
                "TYPE" => r#type = Some(child.text()),
                _ => (),
            }
        }

        let image = match (r#type, binval, extval) {
            (_, None, None) => {
                return Err(ParseError::Generic {
                    msg: "Missing BINVAL or EXTVAL in PHOTO.".to_string(),
                })
            }
            (None, Some(_), None) => {
                return Err(ParseError::Generic {
                    msg: "Missing TYPE in PHOTO.".to_string(),
                })
            }
            (Some(r#type), Some(binval), _) => Image::Binary(r#type, AvatarData::Base64(binval)),
            (_, _, Some(extval)) => Image::External(extval),
        };

        Ok(image)
    }
}

impl FromStr for AddressType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "HOME" => AddressType::Home,
            "WORK" => AddressType::Work,
            "POSTAL" => AddressType::Postal,
            "PARCEL" => AddressType::Parcel,
            "DOM" => AddressType::Dom,
            "INTL" => AddressType::Intl,
            "PREF" => AddressType::Pref,
            _ => {
                return Err(ParseError::Generic {
                    msg: format!("Encountered unexpected AddressType {s}"),
                })
            }
        })
    }
}

impl TryFrom<&Element> for Address {
    type Error = ParseError;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        element.expect_is("ADR", ns::VCARD)?;

        let mut address = Address::default();

        for child in element.children() {
            match child.name() {
                "POBOX" => address.po_box = child.non_empty_text(),
                "EXTADD" => address.ext_add = child.non_empty_text(),
                "STREET" => address.street = child.non_empty_text(),
                "LOCALITY" => address.locality = child.non_empty_text(),
                "REGION" => address.region = child.non_empty_text(),
                "PCODE" => address.postal_code = child.non_empty_text(),
                "CTRY" => address.country = child.non_empty_text(),
                _ => {
                    if let Ok(r#type) = child.name().parse() {
                        address.types.push(r#type);
                    }
                }
            }
        }

        Ok(address)
    }
}

impl TryFrom<&Element> for Label {
    type Error = ParseError;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        element.expect_is("LABEL", ns::VCARD)?;

        let mut label = Label::default();

        for child in element.children() {
            match child.name() {
                "LINE" => label.lines.push(child.text()),
                _ => {
                    if let Ok(r#type) = child.name().parse() {
                        label.types.push(r#type);
                    }
                }
            }
        }

        Ok(label)
    }
}

impl FromStr for TelType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "HOME" => TelType::Home,
            "WORK" => TelType::Work,
            "VOICE" => TelType::Voice,
            "FAX" => TelType::Fax,
            "PAGER" => TelType::Pager,
            "MSG" => TelType::Msg,
            "CELL" => TelType::Cell,
            "VIDEO" => TelType::Video,
            "BBS" => TelType::Bbs,
            "MODEM" => TelType::Modem,
            "ISDN" => TelType::Isdn,
            "PCS" => TelType::Pcs,
            "PREF" => TelType::Pref,
            _ => {
                return Err(ParseError::Generic {
                    msg: format!("Encountered unexpected TelType {s}"),
                })
            }
        })
    }
}

impl TryFrom<&Element> for Telephone {
    type Error = ParseError;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        element.expect_is("TEL", ns::VCARD)?;

        let mut telephone = Telephone::default();

        for child in element.children() {
            match child.name() {
                "NUMBER" => telephone.number = child.text(),
                _ => {
                    if let Ok(r#type) = child.name().parse() {
                        telephone.types.push(r#type);
                    }
                }
            }
        }

        Ok(telephone)
    }
}

impl FromStr for EmailType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "HOME" => EmailType::Home,
            "WORK" => EmailType::Work,
            "INTERNET" => EmailType::Internet,
            "PREF" => EmailType::Pref,
            "X400" => EmailType::X400,
            _ => {
                return Err(ParseError::Generic {
                    msg: format!("Encountered unexpected EmailType {s}"),
                })
            }
        })
    }
}

impl TryFrom<&Element> for Email {
    type Error = ParseError;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        element.expect_is("EMAIL", ns::VCARD)?;

        let mut userid = None;
        let mut types = vec![];

        for child in element.children() {
            match child.name() {
                "USERID" => userid = Some(child.text()),
                _ => {
                    if let Ok(r#type) = child.name().parse() {
                        types.push(r#type);
                    }
                }
            }
        }

        let Some(userid) = userid else {
            return Err(ParseError::Generic {
                msg: "Missing USERID element".to_string(),
            });
        };

        Ok(Email { types, userid })
    }
}

impl TryFrom<&Element> for Geo {
    type Error = ParseError;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        let lat: f64 = element
            .get_child("LAT", ns::VCARD)
            .ok_or_else(|| ParseError::Generic {
                msg: "Missing LAT element".to_string(),
            })?
            .text()
            .parse()
            .map_err(|_| ParseError::Generic {
                msg: "Invalid LAT value".to_string(),
            })?;
        let lon: f64 = element
            .get_child("LON", ns::VCARD)
            .ok_or_else(|| ParseError::Generic {
                msg: "Missing LON element".to_string(),
            })?
            .text()
            .parse()
            .map_err(|_| ParseError::Generic {
                msg: "Invalid LON value".to_string(),
            })?;
        Ok(Geo { lat, lon })
    }
}

impl TryFrom<&Element> for Agent {
    type Error = ParseError;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        let agent_type = element.attr("TYPE").unwrap_or_default().to_string();
        if agent_type == "VCARD" {
            let vcard = Box::new(VCard::try_from(element)?);
            Ok(Agent::VCard(vcard))
        } else {
            let uri = element.text();
            Ok(Agent::External(uri))
        }
    }
}

impl TryFrom<&Element> for Organization {
    type Error = ParseError;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        element.expect_is("ORG", ns::VCARD)?;

        let mut name = None;
        let mut units = vec![];

        for child in element.children() {
            match child.name() {
                "ORGNAME" => name = Some(child.text()),
                "ORGUNIT" => {
                    if let Some(unit) = child.non_empty_text() {
                        units.push(unit);
                    }
                }
                _ => (),
            }
        }

        let Some(name) = name else {
            return Err(ParseError::Generic {
                msg: "Missing ORGNAME element".to_string(),
            });
        };

        Ok(Organization { name, units })
    }
}

impl TryFrom<&Element> for Sound {
    type Error = ParseError;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        let encoding_type = element.attr("ENCODING");
        if let Some("BASE64") = encoding_type {
            let data = element.text();
            Ok(Sound::Binary(data))
        } else if let Some("PHONETIC") = encoding_type {
            let data = element.text();
            Ok(Sound::Phonetic(data))
        } else {
            let uri = element.text();
            Ok(Sound::External(uri))
        }
    }
}

impl FromStr for Class {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "PUBLIC" => Class::Public,
            "PRIVATE" => Class::Private,
            "CONFIDENTIAL" => Class::Confidential,
            _ => {
                return Err(ParseError::Generic {
                    msg: format!("Encountered unexpected Class {s}"),
                })
            }
        })
    }
}

impl TryFrom<&Element> for Key {
    type Error = ParseError;

    fn try_from(element: &Element) -> Result<Self, Self::Error> {
        let type_ = element.attr("TYPE").map(String::from);
        let cred = element.text();
        Ok(Key { type_, cred })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_deserialize_vcard() -> Result<()> {
        let xml = r#"<vCard xmlns='vcard-temp'>
            <FN>Peter Saint-Andre</FN>
            <N>
              <FAMILY>Saint-Andre</FAMILY>
              <GIVEN>Peter</GIVEN>
              <MIDDLE/>
            </N>
            <NICKNAME>stpeter</NICKNAME>
            <URL>http://www.xmpp.org/xsf/people/stpeter.shtml</URL>
            <BDAY>1966-08-06</BDAY>
            <ORG>
              <ORGNAME>XMPP Standards Foundation</ORGNAME>
              <ORGUNIT/>
            </ORG>
            <TITLE>Executive Director</TITLE>
            <ROLE>Patron Saint</ROLE>
            <TEL><WORK/><VOICE/><NUMBER>303-308-3282</NUMBER></TEL>
            <TEL><WORK/><FAX/><NUMBER/></TEL>
            <TEL><WORK/><MSG/><NUMBER/></TEL>
            <ADR>
              <WORK/>
              <EXTADD>Suite 600</EXTADD>
              <STREET>1899 Wynkoop Street</STREET>
              <LOCALITY>Denver</LOCALITY>
              <REGION>CO</REGION>
              <PCODE>80202</PCODE>
              <CTRY>USA</CTRY>
            </ADR>
            <TEL><HOME/><VOICE/><NUMBER>303-555-1212</NUMBER></TEL>
            <TEL><HOME/><FAX/><NUMBER/></TEL>
            <TEL><HOME/><MSG/><NUMBER/></TEL>
            <ADR>
              <HOME/>
              <EXTADD/>
              <STREET/>
              <LOCALITY>Denver</LOCALITY>
              <REGION>CO</REGION>
              <PCODE>80209</PCODE>
              <CTRY>USA</CTRY>
            </ADR>
            <EMAIL><INTERNET/><PREF/><USERID>stpeter@jabber.org</USERID></EMAIL>
            <JABBERID>stpeter@jabber.org</JABBERID>
            <PHOTO>
                <TYPE>image/jpeg</TYPE>
                <BINVAL>Zm9vCg==</BINVAL>
            </PHOTO>
            <DESC>More information about me is located on my personal website: http://www.saint-andre.com/</DESC>
          </vCard>"#;

        assert_eq!(
            VCard {
                fn_: Some("Peter Saint-Andre".to_string()),
                n: Some(Name {
                    family: Some("Saint-Andre".to_string()),
                    given: Some("Peter".to_string()),
                    middle: None,
                    prefix: None,
                    suffix: None,
                }),
                nickname: Some("stpeter".to_string()),
                photo: Some(Image::Binary("image/jpeg".to_string(), AvatarData::Base64("Zm9vCg==".to_string()))),
                bday: Some("1966-08-06".to_string()),
                adr: vec![
                    Address {
                        types: vec![AddressType::Work],
                        po_box: None,
                        ext_add: Some("Suite 600".to_string()),
                        street: Some("1899 Wynkoop Street".to_string()),
                        locality: Some("Denver".to_string()),
                        region: Some("CO".to_string()),
                        postal_code: Some("80202".to_string()),
                        country: Some("USA".to_string()),
                    },
                    Address {
                        types: vec![AddressType::Home],
                        po_box: None,
                        ext_add: None,
                        street: None,
                        locality: Some("Denver".to_string()),
                        region: Some("CO".to_string()),
                        postal_code: Some("80209".to_string()),
                        country: Some("USA".to_string()),
                    }
                ],
                label: vec![],
                tel: vec![
                    Telephone {
                        types: vec![TelType::Work, TelType::Voice],
                        number: "303-308-3282".to_string(),
                    },
                    Telephone {
                        types: vec![TelType::Home, TelType::Voice],
                        number: "303-555-1212".to_string(),
                    }
                ],
                email: vec![Email {
                    types: vec![EmailType::Internet, EmailType::Pref],
                    userid: "stpeter@jabber.org".to_string(),
                }],
                jabberid: Some("stpeter@jabber.org".to_string()),
                mailer: None,
                tz: None,
                geo: None,
                title: Some("Executive Director".to_string()),
                role: Some("Patron Saint".to_string()),
                logo: None,
                agent: None,
                org: Some(Organization {
                    name: "XMPP Standards Foundation".to_string(),
                    units: vec![],
                }),
                categories: vec![],
                note: None,
                prodid: None,
                rev: None,
                sort_string: None,
                sound: None,
                uid: None,
                url: Some("http://www.xmpp.org/xsf/people/stpeter.shtml".to_string()),
                class: None,
                key: None,
                desc: Some("More information about me is located on my personal website: http://www.saint-andre.com/".to_string()),
            },
            VCard::try_from(&Element::from_str(xml)?)?
        );

        Ok(())
    }
}
