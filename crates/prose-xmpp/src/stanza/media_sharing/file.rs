// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::anyhow;
use minidom::{Element, NSChoice};
use xmpp_parsers::hashes::Hash;

use crate::stanza::media_sharing::Thumbnail;
use crate::{ns, ElementExt};

/// The file element is the same as from Jingle File Transfer (XEP-0234) [2].
/// It MUST specify media-type, size, description, and one or multiple hash elements as described
/// in Use of Cryptographic Hash Functions in XMPP (XEP-0300) [14]. The hash elements are essential
/// as they provide end-to-end file integrity and allow efficient caching and flexible
/// retrieval methods.
/// https://xmpp.org/extensions/xep-0385.html#usecases-sending-photo
#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub media_type: String,
    pub name: Option<String>,
    pub size: u64,
    pub desc: Option<String>,
    pub duration: Option<u64>,
    pub hashes: Vec<Hash>,
    pub thumbnails: Vec<Thumbnail>,
}

impl TryFrom<Element> for File {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        let jingle_ft_ns = NSChoice::AnyOf(&[ns::JINGLE_FT, ns::JINGLE_FT_4]);

        value.expect_is("file", jingle_ft_ns)?;

        let mut media_type: Option<String> = None;
        let mut name: Option<String> = None;
        let mut size: Option<u64> = None;
        let mut desc: Option<String> = None;
        let mut duration: Option<u64> = None;
        let mut hashes = vec![];
        let mut thumbnails = vec![];

        for child in value.children() {
            match child {
                _ if child.is("media-type", jingle_ft_ns) => media_type = Some(child.text()),
                _ if child.is("name", jingle_ft_ns) => name = Some(child.text()),
                _ if child.is("size", jingle_ft_ns) => size = Some(child.text().parse()?),
                _ if child.is("hash", ns::HASHES) => hashes.push(Hash::try_from(child.clone())?),
                _ if child.is("desc", jingle_ft_ns) => {
                    let text = child.text();
                    desc = (!text.is_empty()).then_some(text);
                }
                _ if child.is("duration", ns::PROSE_AUDIO_DURATION) => {
                    duration = Some(child.text().parse()?)
                }
                _ if child.is("thumbnail", ns::JINGLE_THUMBS) => {
                    thumbnails.push(Thumbnail::try_from(child.clone())?)
                }
                _ => (),
            }
        }

        let Some(media_type) = media_type else {
            return Err(anyhow!("'media-type' element missing in 'file'."));
        };

        let Some(size) = size else {
            return Err(anyhow!("'size' element missing in 'file'."));
        };

        Ok(File {
            media_type,
            name,
            size,
            desc,
            duration,
            hashes,
            thumbnails,
        })
    }
}

impl From<File> for Element {
    fn from(value: File) -> Self {
        let jingle_ft_ns = ns::JINGLE_FT;

        Element::builder("file", jingle_ft_ns)
            .append(Element::builder("media-type", jingle_ft_ns).append(value.media_type))
            .append_all(
                value
                    .name
                    .map(|name| Element::builder("name", jingle_ft_ns).append(name)),
            )
            .append(Element::builder("size", jingle_ft_ns).append(value.size.to_string()))
            .append_all(
                value
                    .desc
                    .map(|desc| Element::builder("desc", jingle_ft_ns).append(desc)),
            )
            .append_all(value.duration.map(|dur| {
                Element::builder("duration", ns::PROSE_AUDIO_DURATION).append(dur.to_string())
            }))
            .append_all(value.hashes)
            .append_all(value.thumbnails)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Result;
    use xmpp_parsers::hashes::Algo;

    use super::*;

    #[test]
    fn test_deserialize_file() -> Result<()> {
        let xml = r#"<file xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
            <media-type>image/jpeg</media-type>
            <name>summit.jpg</name>
            <size>3032449</size>
            <hash xmlns='urn:xmpp:hashes:2' algo='sha3-256'>2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=</hash>
            <desc>Photo from the summit.</desc>
            <thumbnail xmlns='urn:xmpp:thumbs:1' uri='cid:sha1+ffd7c8d28e9c5e82afea41f97108c6b4@bob.xmpp.org' media-type='image/png' width='128' height='96'/>
        </file>
        "#;

        let elem = Element::from_str(xml)?;
        let file = File::try_from(elem)?;

        assert_eq!(
            file,
            File {
                media_type: "image/jpeg".to_string(),
                name: Some("summit.jpg".to_string()),
                size: 3032449,
                desc: Some("Photo from the summit.".to_string()),
                duration: None,
                hashes: vec![Hash::from_base64(
                    Algo::Sha3_256,
                    "2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU="
                )?,],
                thumbnails: vec![Thumbnail {
                    uri: "cid:sha1+ffd7c8d28e9c5e82afea41f97108c6b4@bob.xmpp.org".to_string(),
                    media_type: Some("image/png".to_string()),
                    width: Some(128),
                    height: Some(96),
                }],
            }
        );

        Ok(())
    }

    #[test]
    fn test_deserialize_proose_audio_file() -> Result<()> {
        let xml = r#"<file xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
            <media-type>audio/aac</media-type>
            <name>audio.aac</name>
            <size>12345</size>
            <duration xmlns='https://prose.org/protocol/audio-duration'>120</duration>
        </file>
        "#;

        let elem = Element::from_str(xml)?;
        let file = File::try_from(elem)?;

        assert_eq!(
            file,
            File {
                media_type: "audio/aac".to_string(),
                name: Some("audio.aac".to_string()),
                size: 12345,
                desc: None,
                duration: Some(120),
                hashes: vec![],
                thumbnails: vec![],
            }
        );

        Ok(())
    }

    #[test]
    fn test_deserialize_file_with_empty_desc() -> Result<()> {
        let xml = r#"<file xmlns='urn:xmpp:jingle:apps:file-transfer:5'>
            <media-type>image/jpeg</media-type>
            <size>3032449</size>
            <hash xmlns='urn:xmpp:hashes:2' algo='sha3-256'>2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=</hash>
            <desc/>
        </file>
        "#;

        let elem = Element::from_str(xml)?;
        let file = File::try_from(elem)?;

        assert_eq!(
            file,
            File {
                media_type: "image/jpeg".to_string(),
                name: None,
                size: 3032449,
                desc: None,
                duration: None,
                hashes: vec![Hash::from_base64(
                    Algo::Sha3_256,
                    "2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU="
                )?,],
                thumbnails: vec![],
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize_file() -> Result<()> {
        let file = File {
            media_type: "image/jpeg".to_string(),
            name: Some("summit.jpg".to_string()),
            size: 3032449,
            desc: Some("Photo from the summit.".to_string()),
            duration: None,
            hashes: vec![Hash::from_base64(
                Algo::Sha3_256,
                "2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=",
            )?],
            thumbnails: vec![Thumbnail {
                uri: "cid:sha1+ffd7c8d28e9c5e82afea41f97108c6b4@bob.xmpp.org".to_string(),
                media_type: Some("image/png".to_string()),
                width: Some(128),
                height: Some(96),
            }],
        };

        let element = Element::try_from(file.clone())?;
        let parsed_file = File::try_from(element)?;

        assert_eq!(parsed_file, file);
        Ok(())
    }

    #[test]
    fn test_serialize_prose_audio_file() -> Result<()> {
        let file = File {
            media_type: "audio/aac".to_string(),
            name: Some("audio.aac".to_string()),
            size: 12345,
            desc: None,
            duration: Some(120),
            hashes: vec![],
            thumbnails: vec![],
        };

        let element = Element::try_from(file.clone())?;
        let parsed_file = File::try_from(element)?;

        assert_eq!(parsed_file, file);
        Ok(())
    }

    #[test]
    fn test_serialize_file_with_empty_desc() -> Result<()> {
        let file = File {
            media_type: "image/jpeg".to_string(),
            name: None,
            size: 3032449,
            desc: None,
            duration: None,
            hashes: vec![Hash::from_base64(
                Algo::Sha3_256,
                "2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=",
            )?],
            thumbnails: vec![],
        };

        let element = Element::try_from(file.clone())?;
        let parsed_file = File::try_from(element)?;

        assert_eq!(parsed_file, file);
        Ok(())
    }
}
