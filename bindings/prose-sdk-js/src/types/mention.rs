// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Range;

use wasm_bindgen::prelude::wasm_bindgen;

use prose_core_client::dtos::Utf16Index;

use crate::types::BareJid;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Mention {
    pub(crate) user: BareJid,
    pub(crate) range: Range<Utf16Index>,
}

#[wasm_bindgen]
impl Mention {
    /// Constructs a new `Mention`.
    ///
    /// # Arguments
    ///
    /// * `user` - BareJID of the user being mentioned.
    /// * `start` - JS index indicating start of the mention in the source string.
    /// * `end` - JS index indicating end of the mention in the source string.
    ///
    /// # Panics
    ///
    /// Panics if `start` is not less than `end`.
    #[wasm_bindgen(constructor)]
    pub fn new(user: BareJid, start: usize, end: usize) -> Self {
        assert!(
            start < end,
            "Cannot construct 'Mention'. 'end' must be greater than 'start'"
        );
        Self {
            user,
            range: Utf16Index::new(start)..Utf16Index::new(end),
        }
    }

    /// Gets the JID of the user being mentioned.
    ///
    /// Returns a duplicate of the user's `BareJid`.
    #[wasm_bindgen(getter)]
    pub fn user(&self) -> BareJid {
        self.user.clone()
    }

    /// Gets the start index of the mention in the source string, based on Javascript's UTF-16
    /// string indexes.
    #[wasm_bindgen(getter)]
    pub fn start(&self) -> usize {
        *self.range.start.as_ref()
    }

    /// Gets the end index of the mention in the source string, based on Javascript's UTF-16
    /// string indexes.
    #[wasm_bindgen(getter)]
    pub fn end(&self) -> usize {
        *self.range.end.as_ref()
    }
}
