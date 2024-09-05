// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use crate::domain::shared::models::{AvatarId, ParticipantId, ParticipantIdRef, UserId};
use crate::domain::user_info::models::AvatarMetadata;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AvatarSource {
    Pep {
        owner: UserId,
        mime_type: String,
    },
    Vcard {
        owner: ParticipantId,
        real_id: Option<UserId>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Avatar {
    pub id: AvatarId,
    pub source: AvatarSource,
}

impl Avatar {
    pub fn from_metadata(user_id: UserId, metadata: AvatarMetadata) -> Self {
        Self {
            id: metadata.checksum,
            source: AvatarSource::Pep {
                owner: user_id,
                mime_type: metadata.mime_type,
            },
        }
    }
}

impl Avatar {
    pub fn owner(&self) -> ParticipantIdRef {
        match &self.source {
            AvatarSource::Pep { owner, .. } => owner.into(),
            AvatarSource::Vcard { owner, .. } => owner.to_ref(),
        }
    }

    pub fn real_id(&self) -> Option<UserId> {
        match &self.source {
            AvatarSource::Pep { .. } => None,
            AvatarSource::Vcard {
                real_id: Some(real_id),
                ..
            } => Some(real_id.clone()),
            AvatarSource::Vcard { .. } => None,
        }
    }

    pub fn is_pep(&self) -> bool {
        match &self.source {
            AvatarSource::Pep { .. } => true,
            AvatarSource::Vcard { .. } => false,
        }
    }
}
