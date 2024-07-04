// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::borrow::Cow;

use crate::domain::shared::models::{OccupantId, ParticipantId, UserId};

#[derive(Debug)]
pub struct ContactNameBuilder<'a> {
    value: Option<Cow<'a, str>>,
}

impl<'a> ContactNameBuilder<'a> {
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl<'a> ContactNameBuilder<'a> {
    pub fn or_firstname_lastname<S: AsRef<str> + 'a>(
        self,
        first_name: Option<&S>,
        last_name: Option<&S>,
    ) -> Self {
        Self {
            value: self
                .value
                .or_else(|| concatenate_names(first_name, last_name).map(Cow::Owned)),
        }
    }

    pub fn or_nickname<S: AsRef<str> + 'a>(self, nickname: Option<&'a S>) -> Self {
        Self {
            value: self
                .value
                .or_else(|| nickname.map(|n| Cow::Borrowed(n.as_ref()))),
        }
    }

    pub fn or_username(self, user_id: Option<&UserId>) -> Self {
        Self {
            value: self
                .value
                .or_else(|| user_id.map(|id| Cow::Owned(id.formatted_username()))),
        }
    }
}

impl<'a> ContactNameBuilder<'a> {
    pub fn unwrap_or_username(self, user_id: &UserId) -> String {
        self.build().unwrap_or_else(|| user_id.formatted_username())
    }

    pub fn unwrap_or_occupant_nickname(self, occupant_id: &OccupantId) -> String {
        self.build()
            .unwrap_or_else(|| occupant_id.formatted_nickname())
    }

    pub fn unwrap_or_participant_id(self, participant_id: &ParticipantId) -> String {
        match participant_id {
            ParticipantId::User(id) => self.unwrap_or_username(id),
            ParticipantId::Occupant(id) => self.unwrap_or_occupant_nickname(id),
        }
    }

    pub fn build(self) -> Option<String> {
        self.value.map(|value| value.to_string())
    }
}

fn concatenate_names<S: AsRef<str>>(
    first_name: Option<&S>,
    last_name: Option<&S>,
) -> Option<String> {
    let parts = first_name
        .into_iter()
        .chain(last_name.into_iter())
        .map(|s| s.as_ref())
        .collect::<Vec<&str>>();

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}
