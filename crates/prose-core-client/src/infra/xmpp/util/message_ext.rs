// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
use tracing::error;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::{eme, legacy_omemo};

use prose_xmpp::ns;
use prose_xmpp::stanza::media_sharing::{MediaShare, OOB};
use prose_xmpp::stanza::Message;

use crate::domain::messaging::models::send_message_request::{Body, Payload};
use crate::domain::messaging::models::Attachment;
use crate::domain::shared::models::UserEndpointId;
use crate::dtos::{MessageServerId, RoomId};

pub trait MessageExt {
    /// Returns unique attachments. Either SIMS or OOB.
    fn attachments(&self) -> Vec<Attachment>;

    /// Appends the given attachments by adding a media-sharing and an OOB element for each.
    fn append_attachments(&mut self, attachments: Vec<Attachment>);

    /// Returns 'true' if the message is a groupchat message which can be either the case if
    /// its type is 'groupchat' or if it contains an element "<x xmlns='http://jabber.org/protocol/muc#user' />".
    /// The latter can happen even for 'chat' messages, e.g. for private messages in a MUC room.
    fn is_groupchat_message(&self) -> bool;
    fn set_message_body(self, body: Option<Body>) -> Self;
    fn set_omemo_payload(self, payload: impl Into<legacy_omemo::Encrypted>) -> Self;

    /// Returns the value of the `from` attribute converted to a `UserEndpointId`, depending on
    /// the message type (groupchat or chat).
    fn sender(&self) -> Option<UserEndpointId>;

    /// Returns the value of the `to` attribute converted to a `RoomId`, depending on the
    /// message type (groupchat or chat)
    fn room_id(&self) -> Option<RoomId>;

    /// Returns the value of the `stanza-id` element converted to a `MessageServerId`.
    fn server_id(&self) -> Option<MessageServerId>;
}

impl MessageExt for Message {
    fn attachments(&self) -> Vec<Attachment> {
        let mut attachments = Vec::<Attachment>::new();

        let mut push_attachment_if_needed = |attachment: Attachment| {
            if attachments
                .iter()
                .find(|a| a.url == attachment.url)
                .is_some()
            {
                return;
            }

            attachments.push(attachment)
        };

        for share in self.media_shares() {
            let attachment = match Attachment::try_from(share) {
                Ok(attachment) => attachment,
                Err(err) => {
                    error!(
                        "Failed to convert media-share to Attachment. {}",
                        err.to_string()
                    );
                    continue;
                }
            };
            push_attachment_if_needed(attachment)
        }

        for oob in self.oob_attachments() {
            let attachment = match Attachment::try_from(oob) {
                Ok(attachment) => attachment,
                Err(err) => {
                    error!("Encountered invalid oob element. {}", err.to_string());
                    continue;
                }
            };
            push_attachment_if_needed(attachment)
        }

        attachments
    }

    fn append_attachments(&mut self, attachments: Vec<Attachment>) {
        for attachment in attachments {
            let reference = Element::builder("reference", ns::REFERENCE)
                .attr("type", "data")
                .append(MediaShare::from(attachment.clone()))
                .build();

            self.payloads.push(reference);
            self.payloads.push(OOB::from(attachment).into())
        }
    }

    fn is_groupchat_message(&self) -> bool {
        if self.type_ == MessageType::Groupchat {
            return true;
        }
        self.payloads
            .iter()
            .find(|p| p.is("x", ns::MUC_USER))
            .is_some()
    }

    fn set_message_body(mut self, body: Option<Body>) -> Self {
        let Some(body) = body else {
            return self;
        };

        self = self.add_references(body.mentions.into_iter().map(Into::into));

        match body.payload {
            Payload::Unencrypted { message, fallback } => self
                .add_content("text/markdown", message.into_string())
                .set_body(fallback.into_string()),
            Payload::Encrypted(encrypted_payload) => self
                .set_omemo_payload(encrypted_payload)
                .set_body("[This message is OMEMO encrypted]"),
        }
    }

    fn set_omemo_payload(mut self, payload: impl Into<legacy_omemo::Encrypted>) -> Self {
        self.payloads.push(Element::from(payload.into()));
        self.payloads.push(
            eme::ExplicitMessageEncryption {
                namespace: ns::LEGACY_OMEMO.to_string(),
                name: Some("OMEMO".to_string()),
            }
            .into(),
        );
        self
    }

    fn sender(&self) -> Option<UserEndpointId> {
        let Some(from) = self.from.clone() else {
            return None;
        };

        if self.is_groupchat_message() {
            let Ok(from) = from.try_into_full() else {
                error!("Expected FullJid in received groupchat message");
                return None;
            };
            UserEndpointId::Occupant(from.into())
        } else {
            match from.try_into_full() {
                Ok(full) => UserEndpointId::UserResource(full.into()),
                Err(bare) => UserEndpointId::User(bare.into()),
            }
        }
        .into()
    }

    fn room_id(&self) -> Option<RoomId> {
        let Some(to) = self.to.clone() else {
            return None;
        };

        if self.is_groupchat_message() {
            Some(RoomId::Muc(to.into_bare().into()))
        } else {
            Some(RoomId::User(to.into_bare().into()))
        }
    }

    fn server_id(&self) -> Option<MessageServerId> {
        self.stanza_id()
            .map(|sid| MessageServerId::from(sid.id.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Result;
    use mime::Mime;
    use url::Url;

    use crate::domain::messaging::models::{AttachmentType, Thumbnail};

    use super::*;

    #[test]
    fn test_returns_unique_attachments() -> Result<()> {
        let xml = r#"<message to='m@nsm.chat' xml:lang='en' type='chat' from='nesium@movim.eu/movim5RPPuJ' id='dc296e78-ebe1-4850-8b80-843972cd8b01' xmlns='jabber:client'>
          <active xmlns='http://jabber.org/protocol/chatstates' />
          <request xmlns='urn:xmpp:receipts' />
          <markable xmlns='urn:xmpp:chat-markers:0' />
          <reference xmlns='urn:xmpp:reference:0' type='data'>
            <media-sharing xmlns='urn:xmpp:sims:1'>
              <file xmlns='urn:xmpp:jingle:apps:file-transfer:4'>
                <media-type>image/jpeg</media-type>
                <name>different_name.jpg</name>
                <size>255286</size>
              </file>
              <sources>
                <reference type='data' uri='https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.jpg' xmlns='urn:xmpp:reference:0' />
              </sources>
            </media-sharing>
          </reference>
          <x xmlns='jabber:x:oob'>
            <url>https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.jpg</url>
          </x>
          <x xmlns='jabber:x:oob'>
            <url>https://upload.prose.org/video.mp4</url>
          </x>
          <origin-id xmlns='urn:xmpp:sid:0' id='dc296e78-ebe1-4850-8b80-843972cd8b01' />
          <body>https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.jpg</body>
          <stanza-id by='m@nsm.chat' xmlns='urn:xmpp:sid:0' id='q8F6tsAB2Y5oQ6uLlEJruT2B' />
        </message>"#;

        let message = Message::try_from(Element::from_str(xml)?)?;

        assert_eq!(
            message.attachments(),
            vec![Attachment {
                r#type: AttachmentType::Image { thumbnail: None },
                url: Url::from_str("https://upload.movim.eu/files/ea644634757a4c90bfad33bbe89e590c2e525d5c/kJi7kSTmOEpB/164492440299900_1vb3qj9.jpg").unwrap(),
                media_type: Mime::from_str("image/jpeg").unwrap(),
                file_name: "different_name.jpg".to_string(),
                file_size: Some(255286),
            }, Attachment {
                r#type: AttachmentType::Video { duration: None, thumbnail: None },
                url: Url::from_str("https://upload.prose.org/video.mp4").unwrap(),
                media_type: Mime::from_str("video/mp4").unwrap(),
                file_name: "video.mp4".to_string(),
                file_size: None,
            }]
        );

        Ok(())
    }

    #[test]
    fn test_appends_attachments() -> Result<()> {
        let mut message = Message::new().set_body("Hello World");

        let attachments = vec![
            Attachment {
                r#type: AttachmentType::Image {
                    thumbnail: Some(Thumbnail {
                        url: "https://uploads.prose.org/file1_thumbnail.jpg"
                            .parse()
                            .unwrap(),
                        media_type: "image/jpeg".parse().unwrap(),
                        width: Some(400),
                        height: Some(200),
                    }),
                },
                url: "https://uploads.prose.org/file1.jpg".parse().unwrap(),
                media_type: "image/jpeg".parse().unwrap(),
                file_name: "file1.jpg".to_string(),
                file_size: Some(12345),
            },
            Attachment {
                r#type: AttachmentType::Video {
                    duration: Some(240),
                    thumbnail: None,
                },
                url: "https://uploads.prose.org/file2.mp4".parse().unwrap(),
                media_type: "video/mp4".parse().unwrap(),
                file_name: "file2.mp4".to_string(),
                file_size: Some(67890),
            },
        ];

        message.append_attachments(attachments.clone());

        assert_eq!(message.oob_attachments().len(), 2);
        assert_eq!(message.media_shares().len(), 2);

        assert_eq!(message.attachments(), attachments);

        Ok(())
    }
}
