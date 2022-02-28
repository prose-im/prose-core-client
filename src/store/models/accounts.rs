// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use mdl::{Cache, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub p1: String,
    pub p2: u32,
}

impl Model for Account {
    fn key(&self) -> String {
        format!("{}:{}", self.p1, self.p2)
    }
}
