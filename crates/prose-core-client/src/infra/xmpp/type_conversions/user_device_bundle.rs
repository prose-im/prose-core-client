// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{anyhow, Context};
use xmpp_parsers::legacy_omemo::{
    Bundle, IdentityKey as XMPPIdentityKey, PreKeyPublic, Prekeys, SignedPreKeyPublic,
    SignedPreKeySignature,
};

use crate::domain::encryption::models::{
    DeviceBundle, DeviceId, IdentityKey, PublicKey, PublicPreKey, PublicSignedPreKey,
};

impl TryFrom<(DeviceId, Bundle)> for DeviceBundle {
    type Error = anyhow::Error;

    fn try_from(value: (DeviceId, Bundle)) -> Result<Self, Self::Error> {
        let (device_id, bundle) = value;

        let signed_pre_key_public = bundle
            .signed_pre_key_public
            .ok_or(anyhow!("Missing SignedPreKey in bundle"))?;

        let signed_pre_key = PublicSignedPreKey {
            id: signed_pre_key_public
                .signed_pre_key_id
                .ok_or(anyhow!("Missing Id in SignedPreKey"))?
                .into(),
            key: PublicKey::try_from(signed_pre_key_public.data.as_slice())
                .context("Invalid Signed PreKey data")?,
            signature: bundle
                .signed_pre_key_signature
                .ok_or(anyhow!("Missing signed PreKey signature in bundle"))?
                .data
                .into(),
        };

        Ok(Self {
            device_id,
            signed_pre_key,
            identity_key: IdentityKey::try_from(
                bundle
                    .identity_key
                    .ok_or(anyhow!("Missing public IdentityKey in bundle"))?
                    .data
                    .as_slice(),
            )
            .context("Invalid public IdentityKey data in bundle")?,
            pre_keys: bundle
                .prekeys
                .ok_or(anyhow!("Missing PreKeys in bundle"))?
                .keys
                .into_iter()
                .map(PublicPreKey::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<DeviceBundle> for Bundle {
    fn from(value: DeviceBundle) -> Self {
        let signed_pre_key_public = SignedPreKeyPublic {
            signed_pre_key_id: Some(value.signed_pre_key.id.into_inner()),
            data: value.signed_pre_key.key.into_inner().into_vec(),
        };

        Self {
            signed_pre_key_public: Some(signed_pre_key_public),
            signed_pre_key_signature: Some(SignedPreKeySignature {
                data: value.signed_pre_key.signature.into_vec(),
            }),
            identity_key: Some(XMPPIdentityKey {
                data: value.identity_key.into_inner().into_vec(),
            }),
            prekeys: Some(Prekeys {
                keys: value.pre_keys.into_iter().map(Into::into).collect(),
            }),
        }
    }
}

impl TryFrom<PreKeyPublic> for PublicPreKey {
    type Error = anyhow::Error;

    fn try_from(value: PreKeyPublic) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.pre_key_id.into(),
            key: PublicKey::try_from(value.data.as_slice()).context("Invalid PreKey data")?,
        })
    }
}

impl From<PublicPreKey> for PreKeyPublic {
    fn from(value: PublicPreKey) -> Self {
        Self {
            pre_key_id: value.id.into_inner(),
            data: value.key.into_inner().into_vec(),
        }
    }
}
