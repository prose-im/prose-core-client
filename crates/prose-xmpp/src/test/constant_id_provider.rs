// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::IDProvider;

pub struct ConstantIDProvider {
    id: String,
}

impl ConstantIDProvider {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl IDProvider for ConstantIDProvider {
    fn new_id(&self) -> String {
        self.id.clone()
    }
}
