// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Modules --

use jid::BareJid;
use once_cell::sync::OnceCell;
use std::sync::Mutex;

use client::{ProseClient, ProseClientBuilder, ProseClientOrigin};

mod broker;
mod protocol;
mod store;
mod types;
mod utils;

pub mod client;

// TODO
//- inspirations
//- aparte: https://github.com/paulfariello/aparte (uses xmpp-parsers)

#[derive(Debug, thiserror::Error)]
pub enum InitializationError {
    #[error("ProseClient was initialized already.")]
    AlreadyInitialized,
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("An unknown error occurred.")]
    Unknown,
}

pub fn prose_initialize(origin: ProseClientOrigin) -> Result<(), InitializationError> {
    if SHARED_CLIENT.get().is_some() {
        return Err(InitializationError::AlreadyInitialized);
    }

    let mut client = ProseClientBuilder::new()
        .app(origin)
        .build()
        .expect("client built")
        .bind()
        .expect("client bound");

    SHARED_CLIENT.set(Mutex::new(client));

    Ok(())
}

pub fn prose_connect(jid: String, password: String) -> Result<BareJid, LoginError> {
    // For now we convert these fancy nested errors into an obfuscated mess until we
    // have a proper error handling system. We'll probably need root-level flat error enums.
    let mut client = ProseClient::shared().lock().unwrap();
    (*client)
        .add(&jid.clone(), &password.clone())
        .map_err(|_| LoginError::Unknown)
}

impl ProseClient {
    fn shared() -> &'static Mutex<ProseClient> {
        SHARED_CLIENT.get().expect("ProseClient is not initialized")
    }
}

static SHARED_CLIENT: OnceCell<Mutex<ProseClient>> = OnceCell::new();

uniffi_macros::include_scaffolding!("ProseCoreFFI");
