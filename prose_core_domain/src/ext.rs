use crate::UserProfile;

impl Default for UserProfile {
    fn default() -> Self {
        UserProfile {
            full_name: None,
            nickname: None,
            org: None,
            title: None,
            email: None,
            tel: None,
            url: None,
            address: None,
        }
    }
}
