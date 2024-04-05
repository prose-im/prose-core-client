// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::legacy_omemo;

use crate::domain::messaging::models::send_message_request::{EncryptedMessage, EncryptedPayload};

impl From<EncryptedPayload> for legacy_omemo::Encrypted {
    fn from(value: EncryptedPayload) -> Self {
        Self {
            header: legacy_omemo::Header {
                sid: value.device_id.into(),
                keys: value
                    .messages
                    .into_iter()
                    .map(legacy_omemo::Key::from)
                    .collect(),
                iv: legacy_omemo::IV {
                    data: value.iv.into_vec(),
                },
            },
            payload: Some(legacy_omemo::Payload {
                data: value.payload.into_vec(),
            }),
        }
    }
}

impl From<legacy_omemo::Encrypted> for EncryptedPayload {
    fn from(value: legacy_omemo::Encrypted) -> Self {
        Self {
            device_id: value.header.sid.into(),
            iv: value.header.iv.data.into(),
            messages: value
                .header
                .keys
                .into_iter()
                .map(EncryptedMessage::from)
                .collect(),
            // TODO: Handle non-existent payload?
            payload: value
                .payload
                .map(|payload| payload.data.into())
                .unwrap_or_default(),
        }
    }
}

impl From<EncryptedMessage> for legacy_omemo::Key {
    fn from(value: EncryptedMessage) -> Self {
        Self {
            rid: value.device_id.into(),
            prekey: value
                .prekey
                .then_some(legacy_omemo::IsPreKey::True)
                .unwrap_or(legacy_omemo::IsPreKey::False),
            data: value.data.into_vec(),
        }
    }
}

impl From<legacy_omemo::Key> for EncryptedMessage {
    fn from(value: legacy_omemo::Key) -> Self {
        Self {
            device_id: value.rid.into(),
            prekey: value.prekey == legacy_omemo::IsPreKey::True,
            data: value.data.into(),
        }
    }
}
