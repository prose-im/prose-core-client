// prose-core-client/prose-sdk-js
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{anyhow, ensure, Context};
pub use encryption_service::{EncryptionService, JsEncryptionService};
use prose_core_client::dtos::{DeviceId, UserId};
pub use signal_repo::SignalRepo;
pub(self) use signal_repo::*;

mod encryption_service;
mod js_compat;
mod signal_repo;

fn try_decode_address(encoded_address: &str) -> anyhow::Result<(UserId, DeviceId)> {
    let mut parts = encoded_address.rsplitn(2, '.');

    let device_id = parts
        .next()
        .ok_or(anyhow!("Invalid address '{encoded_address}'"))?
        .parse::<u32>()
        .with_context(|| format!("Invalid address '{encoded_address}'"))?
        .into();
    let user_id = parts
        .next()
        .ok_or(anyhow!("Invalid address '{encoded_address}'"))?
        .parse()?;

    ensure!(
        parts.next().is_none(),
        "Invalid address '{encoded_address}'"
    );
    Ok((user_id, device_id))
}
