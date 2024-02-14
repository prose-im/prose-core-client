// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::anyhow;
use minidom::{Element, NSChoice};

use crate::stanza::media_sharing::File;
use crate::stanza::references::Reference;
use crate::{ns, ElementExt};

/// XEP-0385: Stateless Inline Media Sharing (SIMS)
/// https://xmpp.org/extensions/xep-0385.html
#[derive(Debug, Clone, PartialEq)]
pub struct MediaShare {
    pub file: File,
    pub sources: Vec<Reference>,
}

impl TryFrom<Element> for MediaShare {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("media-sharing", ns::SIMS)?;

        let mut file: Option<File> = None;
        let mut sources: Vec<Reference> = vec![];

        for child in value.children() {
            match child {
                _ if child.is("file", NSChoice::AnyOf(&[ns::JINGLE_FT, ns::JINGLE_FT_4])) => {
                    if file.is_some() {
                        return Err(anyhow!(
                            "'media-sharing' element contained more than one 'file' element."
                        ));
                    }
                    file = Some(File::try_from(child.clone())?)
                }
                _ if child.is("sources", ns::SIMS) => {
                    for child in child.children() {
                        if !child.is("reference", ns::REFERENCE) {
                            continue;
                        }
                        sources.push(Reference::try_from(child.clone())?);
                    }
                }
                _ => (),
            }
        }

        let Some(file) = file else {
            return Err(anyhow!(
                "Missing 'file' element in 'media-sharing' element."
            ));
        };

        if sources.is_empty() {
            return Err(anyhow!(
                "Missing 'reference' element in 'media-sharing/sources' element."
            ));
        }

        Ok(Self { file, sources })
    }
}

impl From<MediaShare> for Element {
    fn from(value: MediaShare) -> Self {
        Element::builder("media-sharing", ns::SIMS)
            .append(value.file)
            .append(Element::builder("sources", ns::SIMS).append_all(value.sources))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Result;
    use xmpp_parsers::hashes::{Algo, Hash};

    use crate::stanza::references::ReferenceType;

    use super::*;

    #[test]
    fn test_deserialize_media_share() -> Result<()> {
        let xml = r#"<media-sharing xmlns='urn:xmpp:sims:1'>
            <file xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
                <media-type>image/jpeg</media-type>
                <name>summit.jpg</name>
                <size>3032449</size>
                <hash xmlns='urn:xmpp:hashes:2' algo='sha3-256'>2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=</hash>
                <desc>Photo from the summit.</desc>
            </file>
            <sources>
                <reference xmlns='urn:xmpp:reference:0' type='data' uri='https://download.montague.lit/4a771ac1-f0b2-4a4a-9700-f2a26fa2bb67/summit.jpg' />
            </sources>
        </media-sharing>
        "#;

        let elem = Element::from_str(xml)?;
        let share = MediaShare::try_from(elem)?;

        assert_eq!(
            share,
            MediaShare {
                file: File {
                    media_type: "image/jpeg".to_string(),
                    name: Some("summit.jpg".to_string()),
                    size: 3032449,
                    desc: Some("Photo from the summit.".to_string()),
                    duration: None,
                    hashes: vec![Hash::from_base64(
                        Algo::Sha3_256,
                        "2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU="
                    )?],
                    thumbnails: vec![],
                },
                sources: vec![
                    Reference {
                        r#type: ReferenceType::Data,
                        uri: "https://download.montague.lit/4a771ac1-f0b2-4a4a-9700-f2a26fa2bb67/summit.jpg".to_string(),
                        anchor: None,
                        begin: None,
                        end: None,
                    }
                ],
            }
        );

        Ok(())
    }

    #[test]
    fn test_deserialize_movim_media_share() -> Result<()> {
        let xml = r#"<media-sharing xmlns='urn:xmpp:sims:1'>
            <file xmlns='urn:xmpp:jingle:apps:file-transfer:4'>
                <media-type>image/jpeg</media-type>
                <name>164492440299900_1vb3qj9.jpg</name>
                <size>255286</size>
            </file>
            <sources>
                <reference type='data' uri='https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.jpg' xmlns='urn:xmpp:reference:0' />
            </sources>
        </media-sharing>
        "#;

        let elem = Element::from_str(xml)?;
        let share = MediaShare::try_from(elem)?;

        assert_eq!(
            share,
            MediaShare {
                file: File {
                    media_type: "image/jpeg".to_string(),
                    name: Some("164492440299900_1vb3qj9.jpg".to_string()),
                    size: 255286,
                    desc: None,
                    duration: None,
                    hashes: vec![],
                    thumbnails: vec![],
                },
                sources: vec![
                    Reference {
                        r#type: ReferenceType::Data,
                        uri: "https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.jpg".to_string(),
                        anchor: None,
                        begin: None,
                        end: None,
                    }
                ],
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize_media_share() -> Result<()> {
        let share = MediaShare {
            file: File {
                media_type: "image/jpeg".to_string(),
                name: Some("summit.jpg".to_string()),
                size: 3032449,
                desc: Some("Photo from the summit.".to_string()),
                duration: None,
                hashes: vec![Hash::from_base64(
                    Algo::Sha3_256,
                    "2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=",
                )?],
                thumbnails: vec![],
            },
            sources: vec![Reference {
                r#type: ReferenceType::Data,
                uri:
                    "https://download.montague.lit/4a771ac1-f0b2-4a4a-9700-f2a26fa2bb67/summit.jpg"
                        .to_string(),
                anchor: None,
                begin: None,
                end: None,
            }],
        };

        let parsed_share = MediaShare::try_from(Element::try_from(share.clone())?)?;

        assert_eq!(parsed_share, share);
        Ok(())
    }
}
