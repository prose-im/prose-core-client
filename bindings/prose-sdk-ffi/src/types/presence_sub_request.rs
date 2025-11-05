// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{FFIUserId, PresenceSubRequestId};
use prose_core_client::dtos::PresenceSubRequest as CorePresenceSubRequest;

#[derive(uniffi::Record)]
pub struct PresenceSubRequest {
    pub id: PresenceSubRequestId,
    pub name: String,
    pub user_id: FFIUserId,
}

impl From<CorePresenceSubRequest> for PresenceSubRequest {
    fn from(value: CorePresenceSubRequest) -> Self {
        PresenceSubRequest {
            id: value.id.into(),
            name: value.name,
            user_id: value.user_id.into(),
        }
    }
}
