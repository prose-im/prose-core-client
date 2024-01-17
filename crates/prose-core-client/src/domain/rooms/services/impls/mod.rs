// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

pub use room_utils::{build_nickname, ParticipantsVecExt};
pub use rooms_domain_service::{RoomsDomainService, RoomsDomainServiceDependencies};

mod room_utils;
mod rooms_domain_service;
