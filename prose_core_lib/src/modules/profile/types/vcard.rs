use crate::helpers::StanzaCow;
use crate::modules::profile::{Address, Email, Org, Tel};
use crate::stanza::Namespace;
use crate::stanza_base;

pub struct VCard<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> VCard<'a> {
    pub fn new() -> Self {
        let mut stanza = libstrophe::Stanza::new();
        stanza.set_name("vcard").unwrap();
        stanza.set_ns(Namespace::VCard.to_string()).unwrap();
        VCard {
            stanza: stanza.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.first_child().is_none()
    }

    pub fn full_name(&self) -> Option<String> {
        self.child_by_name("fn")?.child_by_name("text")?.text()
    }

    pub fn set_full_name(self, name: impl AsRef<str>) -> Self {
        self.add_child(Stanza::new("fn").add_child(Stanza::new_text_node("text", name)))
    }

    pub fn nickname(&self) -> Option<String> {
        self.child_by_name("nickname")?
            .child_by_name("text")?
            .text()
    }

    pub fn set_nickname(self, nickname: impl AsRef<str>) -> Self {
        self.add_child(Stanza::new("nickname").add_child(Stanza::new_text_node("text", nickname)))
    }

    pub fn url(&self) -> Option<String> {
        self.child_by_name("url")?.child_by_name("uri")?.text()
    }

    pub fn set_url(self, url: impl AsRef<str>) -> Self {
        self.add_child(Stanza::new("url").add_child(Stanza::new_text_node("uri", url)))
    }

    pub fn email(&self) -> Option<Email> {
        self.child_by_name("email").map(|s| s.into())
    }

    pub fn set_email(self, email: Email) -> Self {
        self.add_child(email)
    }

    pub fn tel(&self) -> Option<Tel> {
        self.child_by_name("tel").map(|s| s.into())
    }

    pub fn set_tel(self, tel: Tel) -> Self {
        self.add_child(tel)
    }

    pub fn address(&self) -> Option<Address> {
        self.child_by_name("adr").map(|s| s.into())
    }

    pub fn set_address(self, adr: Address) -> Self {
        self.add_child(adr)
    }

    pub fn org(&self) -> Option<Org> {
        self.child_by_name("org").map(|s| s.into())
    }

    pub fn set_org(self, org: Org) -> Self {
        self.add_child(org)
    }

    pub fn title(&self) -> Option<String> {
        self.child_by_name("title")?.child_by_name("text")?.text()
    }

    pub fn set_title(self, title: impl AsRef<str>) -> Self {
        self.add_child(Stanza::new("title").add_child(Stanza::new_text_node("text", title)))
    }
}

stanza_base!(VCard);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_vcard() {
        let fin = r#"
        <vcard xmlns="urn:ietf:params:xml:ns:vcard-4.0">
          <fn><text>Valerian Saliou</text></fn>
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

        let fin = VCard::from_str(&fin).unwrap();

        assert_eq!(fin.full_name(), Some("Valerian Saliou".to_string()));
        assert_eq!(fin.nickname(), Some("Valerian".to_string()));
        assert_eq!(fin.url(), Some("https://prose.org/".to_string()));
        assert_eq!(
            fin.email().unwrap().value(),
            Some("valerian@prose.org".to_string())
        );
        assert_eq!(
            fin.email().unwrap().parameters().unwrap().types(),
            vec!["home".to_string(), "work".to_string()]
        );
        assert_eq!(
            fin.address().unwrap().locality(),
            Some("Nantes".to_string())
        );
        assert_eq!(
            fin.address().unwrap().country(),
            Some("France, French Republic".to_string())
        );
    }

    #[test]
    fn test_serialize_vcard() -> anyhow::Result<()> {
        let vcard = VCard::new()
            .set_full_name("Valerian Saliou")
            .set_nickname("Valerian")
            .set_url("https://prose.org/")
            .set_email(Email::new("valerian@prose.org"))
            .set_address(
                Address::new()
                    .set_locality("Nantes")
                    .set_country("France, French Republic"),
            );

        assert_eq!(vcard.to_string(), "<vcard xmlns=\"urn:ietf:params:xml:ns:vcard-4.0\"><fn><text>Valerian Saliou</text></fn><nickname><text>Valerian</text></nickname><url><uri>https://prose.org/</uri></url><email><text>valerian@prose.org</text></email><adr><locality>Nantes</locality><country>France, French Republic</country></adr></vcard>".to_string());
        Ok(())
    }
}
