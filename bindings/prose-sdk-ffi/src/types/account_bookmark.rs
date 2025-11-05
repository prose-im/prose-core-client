// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::FFIUserId;
use prose_core_client::AccountBookmark as ProseAccountBookmark;

#[derive(uniffi::Record)]
pub struct AccountBookmark {
    pub user_id: FFIUserId,
    pub is_selected: bool,
}

impl From<ProseAccountBookmark> for AccountBookmark {
    fn from(value: ProseAccountBookmark) -> Self {
        AccountBookmark {
            user_id: value.user_id.into(),
            is_selected: value.is_selected,
        }
    }
}

impl From<AccountBookmark> for ProseAccountBookmark {
    fn from(value: AccountBookmark) -> Self {
        ProseAccountBookmark {
            user_id: value.user_id.into(),
            is_selected: value.is_selected,
        }
    }
}
