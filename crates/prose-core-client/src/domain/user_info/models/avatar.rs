// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use crate::domain::shared::models::{AvatarId, ParticipantId};
use crate::domain::user_info::models::AvatarInfo;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AvatarSource {
    Pep { mime_type: String },
    Vcard,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Avatar {
    pub id: AvatarId,
    pub source: AvatarSource,
    pub owner: ParticipantId,
}

impl Avatar {
    pub fn info(&self) -> Option<AvatarInfo> {
        let AvatarSource::Pep { mime_type } = &self.source else {
            return None;
        };
        Some(AvatarInfo {
            checksum: self.id.clone(),
            mime_type: mime_type.clone(),
        })
    }
}
