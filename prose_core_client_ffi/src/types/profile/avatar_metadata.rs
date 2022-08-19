use crate::error::Error;
use crate::helpers::StanzaExt;
use libstrophe::Stanza;
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq)]
pub struct AvatarMetadata(Vec<AvatarMetadataInfo>);

impl AvatarMetadata {
    pub fn new(info: Vec<AvatarMetadataInfo>) -> Self {
        AvatarMetadata(info)
    }
}

impl TryFrom<&Stanza> for AvatarMetadata {
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        let items: Result<Vec<_>, _> = stanza
            .children()
            .filter(|n| n.name() == Some("info"))
            .map(|n| AvatarMetadataInfo::try_from(n.deref()))
            .collect();
        Ok(AvatarMetadata(items?))
    }
}

impl Deref for AvatarMetadata {
    type Target = Vec<AvatarMetadataInfo>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AvatarMetadata {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AvatarMetadata {
    pub fn into_inner(self) -> Vec<AvatarMetadataInfo> {
        self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct AvatarMetadataInfo {
    pub id: String,
    pub url: Option<String>,
    pub bytes: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub mime_type: Option<String>,
}

impl AvatarMetadataInfo {
    pub fn new(
        id: impl AsRef<str>,
        url: Option<&str>,
        bytes: Option<u32>,
        width: Option<u32>,
        height: Option<u32>,
        mime_type: Option<&str>,
    ) -> Self {
        AvatarMetadataInfo {
            id: id.as_ref().to_string(),
            url: url.map(|o| o.to_string()),
            bytes,
            width,
            height,
            mime_type: mime_type.map(|o| o.to_string()),
        }
    }
}

impl TryFrom<&Stanza> for AvatarMetadataInfo {
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        Ok(AvatarMetadataInfo::new(
            stanza.get_required_attribute("id")?,
            stanza.get_attribute("url"),
            stanza
                .get_attribute("bytes")
                .and_then(|a| a.parse::<u32>().ok()),
            stanza
                .get_attribute("width")
                .and_then(|a| a.parse::<u32>().ok()),
            stanza
                .get_attribute("height")
                .and_then(|a| a.parse::<u32>().ok()),
            stanza.get_attribute("type"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use libstrophe::Stanza;

    use super::*;

    #[test]
    fn test_deserialize_metadata_with_url() {
        let xml = r#"
        <info bytes="23456" height="64" id="357a8123a30844a3aa99861b6349264ba67a5694" type="image/gif" url="http://avatars.example.org/happy.gif" width="48"/>
        "#;

        let stanza = Stanza::from_str(xml);
        let metadata = AvatarMetadataInfo::try_from(&stanza).unwrap();

        assert_eq!(
            metadata,
            AvatarMetadataInfo::new(
                "357a8123a30844a3aa99861b6349264ba67a5694",
                Some("http://avatars.example.org/happy.gif"),
                Some(23456),
                Some(48),
                Some(64),
                Some("image/gif")
            )
        );
    }

    #[test]
    fn test_deserialize_metadata_without_url() {
        let xml = r#"
        <info bytes="23456" height="64" id="357a8123a30844a3aa99861b6349264ba67a5694" type="image/gif" width="48"/>
        "#;

        let stanza = Stanza::from_str(xml);
        let metadata = AvatarMetadataInfo::try_from(&stanza).unwrap();

        assert_eq!(
            metadata,
            AvatarMetadataInfo::new(
                "357a8123a30844a3aa99861b6349264ba67a5694",
                None,
                Some(23456),
                Some(48),
                Some(64),
                Some("image/gif")
            )
        );
    }
}
