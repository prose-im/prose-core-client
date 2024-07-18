// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::domain::shared::models::UnicodeScalarIndex;
use crate::dtos::UserId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mention {
    pub user: UserId,
    pub range: Option<Range<UnicodeScalarIndex>>,
}
