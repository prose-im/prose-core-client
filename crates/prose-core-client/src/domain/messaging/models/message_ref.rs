// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{MessageId, StanzaId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageRef {
    pub message_id: MessageId,
    pub stanza_id: StanzaId,
    pub timestamp: DateTime<Utc>,
}
