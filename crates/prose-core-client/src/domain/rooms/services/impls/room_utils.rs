use sha1::{Digest, Sha1};

use crate::domain::shared::models::UserId;

const GROUP_PREFIX: &str = "org.prose.group";
const NICKNAME_MAX_LEN: usize = 20;
const NICKNAME_HASH_LEN: usize = 7;

pub fn build_nickname(user_id: &UserId) -> String {
    // We append a suffix to prevent any nickname conflicts, but want to make sure that it is
    // identical between multiple sessions so that these would be displayed as one user.

    let mut hasher = Sha1::new();
    hasher.update(user_id.to_string().to_lowercase());
    let hash = format!("{:x}", hasher.finalize());

    let username = user_id.username();
    let username_max_length = username
        .chars()
        .count()
        .min(NICKNAME_MAX_LEN - NICKNAME_HASH_LEN - 1);

    format!(
        "{}#{}",
        username
            .chars()
            .take(username_max_length)
            .collect::<String>(),
        &hash[hash.len() - NICKNAME_HASH_LEN..]
    )
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
        assert_eq!(build_nickname(&user_id!("user@prose.org")), "user#1ed8798");
        assert_eq!(
            build_nickname(&user_id!("super-long-username@prose.org")),
            "super-long-u#fac4746"
        );
        assert_eq!(build_nickname(&user_id!("josé@prose.org")), "josé#6c09790");
        assert_eq!(
            build_nickname(&user_id!("JoséAndrés123@prose.org")),
            "joséandrés12#7ebd867"
        );
        assert_eq!(
            build_nickname(&user_id!("TwelveCharsé@prose.org")),
            "twelvecharsé#3246eb5"
        );
    }
}
