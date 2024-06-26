// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Formatter};

use base64::{engine::general_purpose, Engine as _};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublicKey(Box<[u8]>);

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PrivateKey(Box<[u8]>);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdentityKey(PublicKey);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentityKeyPair {
    pub identity_key: IdentityKey,
    pub private_key: PrivateKey,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PreKeyId(u32);

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SignedPreKeyId(u32);

#[derive(Clone)]
pub struct SignedPreKey {
    pub id: SignedPreKeyId,
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
    pub signature: Box<[u8]>,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct PublicSignedPreKey {
    pub id: SignedPreKeyId,
    pub key: PublicKey,
    pub signature: Box<[u8]>,
}

#[derive(Clone, Debug)]
pub struct PreKey {
    pub id: PreKeyId,
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
}

#[derive(Clone, Debug)]
pub struct PublicPreKey {
    pub id: PreKeyId,
    pub key: PublicKey,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KyberPreKeyId(u32);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KyberPreKey(Box<[u8]>);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SenderKey(Box<[u8]>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionData(Box<[u8]>);

impl IdentityKeyPair {
    pub fn fingerprint(&self) -> String {
        self.identity_key.fingerprint()
    }
}

impl PublicKey {
    pub fn fingerprint(&self) -> String {
        self.0
            .iter()
            .skip(1)
            .map(|b| format!("{:02x}", b))
            .chunks(4)
            .into_iter()
            .map(|word| word.collect::<String>())
            .join(" ")
    }

    pub fn into_inner(self) -> Box<[u8]> {
        self.0
    }
}

impl PrivateKey {
    pub fn into_inner(self) -> Box<[u8]> {
        self.0
    }
}

impl IdentityKey {
    pub fn into_inner(self) -> Box<[u8]> {
        self.0.into_inner()
    }

    pub fn fingerprint(&self) -> String {
        self.0.fingerprint()
    }
}

impl KyberPreKey {
    pub fn into_inner(self) -> Box<[u8]> {
        self.0
    }
}

impl SenderKey {
    pub fn into_inner(self) -> Box<[u8]> {
        self.0
    }
}

impl From<&[u8]> for PublicKey {
    fn from(value: &[u8]) -> Self {
        Self(value.into())
    }
}

impl From<&[u8]> for PrivateKey {
    fn from(value: &[u8]) -> Self {
        Self(value.into())
    }
}

impl From<&[u8]> for IdentityKey {
    fn from(value: &[u8]) -> Self {
        Self(PublicKey::from(value))
    }
}

impl From<Box<[u8]>> for KyberPreKey {
    fn from(value: Box<[u8]>) -> Self {
        Self(value)
    }
}

impl From<Box<[u8]>> for SenderKey {
    fn from(value: Box<[u8]>) -> Self {
        Self(value)
    }
}

impl From<Box<[u8]>> for SessionData {
    fn from(value: Box<[u8]>) -> Self {
        Self(value)
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for PrivateKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for KyberPreKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for SenderKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for SessionData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for IdentityKey {
    fn as_ref(&self) -> &[u8] {
        &self.0.as_ref()
    }
}

impl From<u32> for PreKeyId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl AsRef<u32> for PreKeyId {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl PreKeyId {
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

impl From<u32> for SignedPreKeyId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl AsRef<u32> for SignedPreKeyId {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl SignedPreKeyId {
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

impl From<PublicKey> for IdentityKey {
    fn from(value: PublicKey) -> Self {
        Self(value)
    }
}

impl From<u32> for KyberPreKeyId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl AsRef<u32> for KyberPreKeyId {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl KyberPreKeyId {
    pub fn into_inner(self) -> u32 {
        self.0
    }
}

impl PreKey {
    pub fn into_public_pre_key(self) -> PublicPreKey {
        PublicPreKey {
            id: self.id,
            key: self.public_key,
        }
    }
}

impl SignedPreKey {
    pub fn into_public_signed_pre_key(self) -> PublicSignedPreKey {
        PublicSignedPreKey {
            id: self.id,
            key: self.public_key,
            signature: self.signature,
        }
    }
}

impl Debug for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PublicKey({})",
            general_purpose::STANDARD.encode(self.0.as_ref())
        )
    }
}

impl Debug for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrivateKey([REDACTED])")
    }
}

impl Debug for SignedPreKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignedPreKeyRecord")
            .field("id", &self.id)
            .field("public_key", &self.public_key)
            .field("private_key", &self.private_key)
            .field(
                "signature",
                &general_purpose::STANDARD.encode(&self.signature),
            )
            .field("timestamp", &self.timestamp)
            .finish()
    }
}
