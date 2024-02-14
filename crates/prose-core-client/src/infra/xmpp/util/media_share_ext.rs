// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::error;
use url::Url;

use prose_xmpp::stanza::media_sharing::MediaShare;
use prose_xmpp::stanza::references::ReferenceType;

pub trait MediaShareExt {
    fn first_http_source_url(&self) -> Option<Url>;
}

impl MediaShareExt for MediaShare {
    fn first_http_source_url(&self) -> Option<Url> {
        self.sources
            .iter()
            .find_map(|source| {
                if source.r#type != ReferenceType::Data {
                    return None
                }

                let url = match source.uri.parse::<Url>() {
                    Ok(url) => url,
                    Err(_) => {
                        error!("Encountered invalid uri '{}' in reference element of a media-sharing element.", source.uri);
                        return None
                    }
                };

                if url.scheme() == "https" || url.scheme() == "http" {
                    return Some(url)
                }
                None
            })
    }
}

#[cfg(test)]
mod tests {
    use prose_xmpp::bare;
    use prose_xmpp::stanza::media_sharing::File;
    use prose_xmpp::stanza::references::Reference;

    use super::*;

    #[test]
    fn test_first_http_source_url() {
        assert_eq!(
            MediaShare {
                file: File {
                    media_type: "image/jpeg".to_string(),
                    name: None,
                    size: 100,
                    desc: None,
                    duration: None,
                    hashes: vec![],
                    thumbnails: vec![],
                },
                sources: vec![
                    Reference::mention(bare!("user@prose.org")),
                    Reference::data_reference("https://uploads.prose.org/image.jpg"),
                ]
            }
            .first_http_source_url()
            .unwrap()
            .to_string()
            .as_str(),
            "https://uploads.prose.org/image.jpg"
        );
    }
}
