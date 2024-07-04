// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use crate::domain::shared::models::{AvatarId, ParticipantId, ParticipantIdRef, UserId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AvatarSource {
    Pep { owner: UserId, mime_type: String },
    Vcard { owner: ParticipantId },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Avatar {
    pub id: AvatarId,
    pub source: AvatarSource,
}

impl Avatar {
    pub fn owner(&self) -> ParticipantIdRef {
        match &self.source {
            AvatarSource::Pep { owner, .. } => owner.into(),
            AvatarSource::Vcard { owner } => owner.to_ref(),
        }
    }

    pub fn is_pep(&self) -> bool {
        match &self.source {
            AvatarSource::Pep { .. } => true,
            AvatarSource::Vcard { .. } => false,
        }
    }
}
