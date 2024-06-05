// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::SystemTime;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use libsignal_protocol::{CiphertextMessage, PreKeySignalMessage, ProtocolAddress, SignalMessage};
use tokio::sync::{mpsc, oneshot};

use crate::app::deps::{DynEncryptionKeysRepository, DynRngProvider, DynSessionRepository};
use crate::domain::encryption::models::{
    DecryptionContext, DeviceId, LocalEncryptionBundle, PreKeyBundle, PreKeyId, PreKeyRecord,
    PrivateKey, PublicKey, SignedPreKeyId, SignedPreKeyRecord,
};
use crate::domain::encryption::services::EncryptionService;
use crate::domain::messaging::models::EncryptionKey;
use crate::dtos::UserId;

use super::SignalRepoWrapper;

struct SignalService {
    receiver: mpsc::Receiver<SignalServiceMessage>,
    encryption_keys_repo: DynEncryptionKeysRepository,
    session_repo: DynSessionRepository,
    rng_provider: DynRngProvider,
}
enum SignalServiceMessage {
    ProcessPreKeyBundle {
        user_id: UserId,
        device_id: DeviceId,
        bundle: libsignal_protocol::PreKeyBundle,
        now: SystemTime,
        callback: oneshot::Sender<Result<()>>,
    },

    EncryptKey {
        recipient_id: UserId,
        device_id: DeviceId,
        message: Box<[u8]>,
        now: SystemTime,
        callback: oneshot::Sender<Result<EncryptionKey>>,
    },

    DecryptKey {
        sender_id: UserId,
        device_id: DeviceId,
        encrypted_message: Box<[u8]>,
        is_pre_key: bool,
        decryption_context: DecryptionContext,
        callback: oneshot::Sender<Result<Box<[u8]>>>,
    },
}

impl SignalService {
    async fn handle_message(&mut self, msg: SignalServiceMessage) {
        match msg {
            SignalServiceMessage::ProcessPreKeyBundle {
                user_id,
                device_id,
                bundle,
                now,
                callback,
            } => {
                _ = callback.send(
                    self.process_prekey_bundle(user_id, device_id, bundle, now)
                        .await,
                );
            }
            SignalServiceMessage::EncryptKey {
                recipient_id,
                device_id,
                message,
                now,
                callback,
            } => {
                _ = callback.send(
                    self.encrypt_key(&recipient_id, device_id, message.as_ref(), &now)
                        .await,
                )
            }
            SignalServiceMessage::DecryptKey {
                sender_id,
                device_id,
                encrypted_message,
                is_pre_key,
                decryption_context,
                callback,
            } => {
                _ = callback.send(
                    self.decrypt_key(
                        sender_id,
                        device_id,
                        encrypted_message,
                        is_pre_key,
                        decryption_context,
                    )
                    .await,
                )
            }
        }
    }

    async fn process_prekey_bundle(
        &self,
        user_id: UserId,
        device_id: DeviceId,
        bundle: libsignal_protocol::PreKeyBundle,
        now: SystemTime,
    ) -> Result<()> {
        let signal_store = SignalRepoWrapper::new(
            self.encryption_keys_repo.clone(),
            self.session_repo.clone(),
            None,
        );

        libsignal_protocol::process_prekey_bundle(
            &ProtocolAddress::new(user_id.to_string(), device_id.clone().into()),
            &mut signal_store.clone(),
            &mut signal_store.clone(),
            &bundle,
            now,
            &mut self.rng_provider.rng(),
        )
        .await?;

        Ok(())
    }

    async fn decrypt_key(
        &self,
        sender_id: UserId,
        device_id: DeviceId,
        encrypted_message: Box<[u8]>,
        is_pre_key: bool,
        decryption_context: DecryptionContext,
    ) -> Result<Box<[u8]>> {
        let address = ProtocolAddress::new(sender_id.to_string(), device_id.clone().into());

        let ciphertext_message = if is_pre_key {
            CiphertextMessage::PreKeySignalMessage(PreKeySignalMessage::try_from(
                encrypted_message.as_ref(),
            )?)
        } else {
            CiphertextMessage::SignalMessage(SignalMessage::try_from(encrypted_message.as_ref())?)
        };

        let signal_store = SignalRepoWrapper::new(
            self.encryption_keys_repo.clone(),
            self.session_repo.clone(),
            Some(decryption_context),
        );

        let dek_and_mac = libsignal_protocol::message_decrypt(
            &ciphertext_message,
            &address,
            &mut signal_store.clone(),
            &mut signal_store.clone(),
            &mut signal_store.clone(),
            &mut signal_store.clone(),
            &mut signal_store.clone(),
            &mut self.rng_provider.rng(),
        )
        .await?;

        Ok(dek_and_mac.into_boxed_slice())
    }

    async fn encrypt_key(
        &self,
        user_id: &UserId,
        device_id: DeviceId,
        message: &[u8],
        now: &SystemTime,
    ) -> Result<EncryptionKey> {
        let signal_store = SignalRepoWrapper::new(
            self.encryption_keys_repo.clone(),
            self.session_repo.clone(),
            None,
        );

        let address = ProtocolAddress::new(user_id.to_string(), device_id.clone().into());

        let encrypted_message = libsignal_protocol::message_encrypt(
            &message,
            &address,
            &mut signal_store.clone(),
            &mut signal_store.clone(),
            now.clone(),
        )
        .await?;

        let encrypted_message = match encrypted_message {
            CiphertextMessage::SignalMessage(message) => EncryptionKey {
                device_id,
                is_pre_key: false,
                data: message.serialized().into(),
            },
            CiphertextMessage::PreKeySignalMessage(message) => EncryptionKey {
                device_id,
                is_pre_key: true,
                data: message.serialized().into(),
            },
            CiphertextMessage::SenderKeyMessage(_) | CiphertextMessage::PlaintextContent(_) => {
                unreachable!()
            }
        };

        Ok(encrypted_message)
    }
}

impl SignalService {
    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }
    }
}

#[derive(Clone)]
pub struct SignalServiceHandle {
    sender: mpsc::Sender<SignalServiceMessage>,
    rng_provider: DynRngProvider,
}

impl SignalServiceHandle {
    pub fn new(
        encryption_keys_repo: DynEncryptionKeysRepository,
        session_repo: DynSessionRepository,
        rng_provider: DynRngProvider,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let mut actor = SignalService {
            receiver,
            encryption_keys_repo,
            session_repo,
            rng_provider: rng_provider.clone(),
        };

        // This feels like overkill, but we need to deal with the fact that the Signal store traits
        // are all ?Send.
        // See:
        // - https://github.com/signalapp/libsignal/issues/298
        // - https://github.com/whisperfish/libsignal-service-rs/issues/111
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        std::thread::spawn(move || {
            let local = tokio::task::LocalSet::new();
            local.spawn_local(async move {
                actor.run().await;
            });
            rt.block_on(local);
        });

        Self {
            sender,
            rng_provider,
        }
    }
}

#[async_trait]
impl EncryptionService for SignalServiceHandle {
    async fn generate_local_encryption_bundle(
        &self,
        device_id: DeviceId,
    ) -> Result<LocalEncryptionBundle> {
        let now = Utc::now();
        let mut rng = self.rng_provider.rng();
        let identity_key_pair = libsignal_protocol::IdentityKeyPair::generate(&mut rng);
        let signed_pre_key = libsignal_protocol::KeyPair::generate(&mut rng);
        let signed_pre_key_signature = identity_key_pair
            .private_key()
            .calculate_signature(&signed_pre_key.public_key.serialize(), &mut rng)?;

        let bundle = LocalEncryptionBundle {
            device_id,
            identity_key_pair: (&identity_key_pair).try_into()?,
            signed_pre_key: SignedPreKeyRecord {
                id: SignedPreKeyId::from(0),
                public_key: (&signed_pre_key.public_key).try_into()?,
                private_key: (&signed_pre_key.private_key).try_into()?,
                signature: signed_pre_key_signature,
                timestamp: now.timestamp() as u64,
            },
            pre_keys: (1u32..101)
                .map(|i| {
                    let key_pair = libsignal_protocol::KeyPair::generate(&mut rng);
                    let result = PublicKey::try_from(&key_pair.public_key).and_then(|public_key| {
                        PrivateKey::try_from(&key_pair.private_key)
                            .map(|private_key| (public_key, private_key))
                    });

                    let (public_key, private_key) = match result {
                        Ok(keys) => keys,
                        Err(err) => return Err(err),
                    };

                    Ok(PreKeyRecord {
                        id: PreKeyId::from(i),
                        public_key,
                        private_key,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        };

        Ok(bundle)
    }

    async fn generate_pre_keys_with_ids(&self, ids: Vec<PreKeyId>) -> Result<Vec<PreKeyRecord>> {
        let mut rng = self.rng_provider.rng();
        let pre_keys = ids
            .into_iter()
            .map(|id| {
                let key_pair = libsignal_protocol::KeyPair::generate(&mut rng);
                let result = PublicKey::try_from(&key_pair.public_key).and_then(|public_key| {
                    PrivateKey::try_from(&key_pair.private_key)
                        .map(|private_key| (public_key, private_key))
                });

                let (public_key, private_key) = match result {
                    Ok(keys) => keys,
                    Err(err) => return Err(err),
                };

                Ok(PreKeyRecord {
                    id: PreKeyId::from(id),
                    public_key,
                    private_key,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(pre_keys)
    }

    async fn process_pre_key_bundle(&self, user_id: &UserId, bundle: PreKeyBundle) -> Result<()> {
        let device_id = bundle.device_id.clone();

        let bundle = libsignal_protocol::PreKeyBundle::new(
            0,
            device_id.clone().into(),
            Some((bundle.pre_key.id.into(), (&bundle.pre_key.key).try_into()?)),
            bundle.signed_pre_key.id.into(),
            (&bundle.signed_pre_key.key).try_into()?,
            bundle.signed_pre_key.signature.into_vec(),
            (&bundle.identity_key).try_into()?,
        )?;

        let (send, recv) = oneshot::channel();
        let message = SignalServiceMessage::ProcessPreKeyBundle {
            user_id: user_id.clone(),
            device_id,
            bundle,
            now: SystemTime::now(),
            callback: send,
        };

        self.sender.send(message).await?;
        recv.await.context("Actor task has been killed")?
    }

    async fn encrypt_key(
        &self,
        recipient_id: &UserId,
        device_id: &DeviceId,
        message: &[u8],
        now: &SystemTime,
    ) -> Result<EncryptionKey> {
        let (send, recv) = oneshot::channel();
        let message = SignalServiceMessage::EncryptKey {
            recipient_id: recipient_id.clone(),
            device_id: device_id.clone(),
            message: message.into(),
            now: now.clone(),
            callback: send,
        };

        self.sender.send(message).await?;
        recv.await.context("Actor task has been killed")?
    }

    async fn decrypt_key(
        &self,
        sender_id: &UserId,
        device_id: &DeviceId,
        message: &[u8],
        is_pre_key: bool,
        decryption_context: DecryptionContext,
    ) -> Result<Box<[u8]>> {
        let (send, recv) = oneshot::channel();
        let message = SignalServiceMessage::DecryptKey {
            sender_id: sender_id.clone(),
            device_id: device_id.clone(),
            encrypted_message: message.into(),
            is_pre_key,
            decryption_context,
            callback: send,
        };

        self.sender.send(message).await?;
        recv.await.context("Actor task has been killed")?
    }
}
