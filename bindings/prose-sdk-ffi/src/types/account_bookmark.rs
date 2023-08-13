// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::JID;
use prose_core_client::AccountBookmark as ProseAccountBookmark;

pub struct AccountBookmark {
    pub jid: JID,
    pub is_selected: bool,
}

impl From<ProseAccountBookmark> for AccountBookmark {
    fn from(value: ProseAccountBookmark) -> Self {
        AccountBookmark {
            jid: value.jid.into(),
            is_selected: value.is_selected,
        }
    }
}

impl From<AccountBookmark> for ProseAccountBookmark {
    fn from(value: AccountBookmark) -> Self {
        ProseAccountBookmark {
            jid: value.jid.into(),
            is_selected: value.is_selected,
        }
    }
}
