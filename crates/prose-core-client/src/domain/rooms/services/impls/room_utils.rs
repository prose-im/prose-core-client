use jid::ResourcePart;
use sha1::{Digest, Sha1};

use crate::domain::shared::models::UserId;

const GROUP_PREFIX: &str = "org.prose.group";

pub fn build_nickname(display_name: Option<&str>, user_id: &UserId) -> String {
    // We check if the display_name can be converted into a Jid resource, i.e. that it doesn't
    // contain any invalid characters. If that's the case, we take it or the node of the
    // user_id otherwise.
    display_name
        .and_then(|display_name| {
            let display_name = display_name.trim();
            (!display_name.is_empty()).then_some(display_name)
        })
        .and_then(|display_name| ResourcePart::new(display_name).ok())
        .map(|res| res.to_string())
        .unwrap_or_else(|| user_id.formatted_username())
}

pub trait ParticipantsVecExt {
    fn group_name_hash(&self) -> String;
}

impl ParticipantsVecExt for Vec<UserId> {
    fn group_name_hash(&self) -> String {
        let mut sorted_participant_jids =
            self.iter().map(|jid| jid.to_string()).collect::<Vec<_>>();
        sorted_participant_jids.sort();

        let mut hasher = Sha1::new();
        hasher.update(sorted_participant_jids.join(","));
        format!("{}.{:x}", GROUP_PREFIX, hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use crate::user_id;

    use super::*;

    #[test]
    fn test_group_name_for_participants() {
        assert_eq!(
            vec![
                user_id!("a@prose.org"),
                user_id!("b@prose.org"),
                user_id!("c@prose.org")
            ]
            .group_name_hash(),
            "org.prose.group.7c138d7281db96e0d42fe026a4195c85a7dc2cae".to_string()
        );

        assert_eq!(
            vec![
                user_id!("a@prose.org"),
                user_id!("b@prose.org"),
                user_id!("c@prose.org")
            ]
            .group_name_hash(),
            vec![
                user_id!("c@prose.org"),
                user_id!("a@prose.org"),
                user_id!("b@prose.org")
            ]
            .group_name_hash()
        )
    }

    #[test]
    fn test_build_nickname() {
        assert_eq!(
            "Jane Doe",
            build_nickname(Some("Jane Doe"), &user_id!("user@prose.org")),
        );
        assert_eq!(
            "User",
            build_nickname(Some("Jane Doe 🧋"), &user_id!("user@prose.org")),
        );
        assert_eq!("User", build_nickname(None, &user_id!("user@prose.org")));
        assert_eq!(
            "User",
            build_nickname(Some(""), &user_id!("user@prose.org"))
        );
        assert_eq!(
            "User",
            build_nickname(Some(" "), &user_id!("user@prose.org"))
        );
        assert_eq!(
            "jane",
            build_nickname(Some(" jane "), &user_id!("user@prose.org"))
        );
    }
}
