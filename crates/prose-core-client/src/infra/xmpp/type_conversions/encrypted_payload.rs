// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::legacy_omemo;

use crate::domain::messaging::models::{
    EncryptedMessage, EncryptedPayload, EncryptionKey, KeyTransportPayload,
};
use crate::dtos::DeviceId;

impl From<EncryptedPayload> for legacy_omemo::Encrypted {
    fn from(value: EncryptedPayload) -> Self {
        Self {
            header: legacy_omemo::Header {
                sid: value.device_id.into(),
                keys: value
                    .keys
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

impl From<KeyTransportPayload> for legacy_omemo::Encrypted {
    fn from(value: KeyTransportPayload) -> Self {
        Self {
            header: legacy_omemo::Header {
                sid: value.device_id.into(),
                keys: value
                    .keys
                    .into_iter()
                    .map(legacy_omemo::Key::from)
                    .collect(),
                iv: legacy_omemo::IV {
                    data: value.iv.into_vec(),
                },
            },
            payload: None,
        }
    }
}

impl From<legacy_omemo::Encrypted> for EncryptedMessage {
    fn from(value: legacy_omemo::Encrypted) -> Self {
        let device_id = DeviceId::from(value.header.sid);
        let iv = value.header.iv.data.into_boxed_slice();
        let keys = value
            .header
            .keys
            .into_iter()
            .map(EncryptionKey::from)
            .collect::<Vec<_>>();

        if let Some(payload) = value.payload {
            EncryptedMessage::Message(EncryptedPayload {
                device_id,
                iv,
                keys,
                payload: payload.data.into_boxed_slice(),
            })
        } else {
            EncryptedMessage::KeyTransport(KeyTransportPayload {
                device_id,
                iv,
                keys,
            })
        }
    }
}

impl From<EncryptionKey> for legacy_omemo::Key {
    fn from(value: EncryptionKey) -> Self {
        Self {
            rid: value.device_id.into(),
            prekey: value.is_pre_key,
            data: value.data.into_vec(),
        }
    }
}

impl From<legacy_omemo::Key> for EncryptionKey {
    fn from(value: legacy_omemo::Key) -> Self {
        Self {
            device_id: value.rid.into(),
            is_pre_key: value.prekey,
            data: value.data.into(),
        }
    }
}
