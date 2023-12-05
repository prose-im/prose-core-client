// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::app::dtos::Contact as ContactDTO;
use prose_core_client::app::services::ContactsService;
use prose_core_client::domain::contacts::models::{Contact, Group};
use prose_core_client::domain::shared::models::{Availability, UserId};
use prose_core_client::domain::user_info::models::UserInfo;
use prose_core_client::domain::user_profiles::models::UserProfile;
use prose_core_client::test::MockAppDependencies;
use prose_core_client::user_id;

#[tokio::test]
async fn test_assembles_contact_dto() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.contacts_repo.expect_get_all().times(1).returning(|_| {
        Box::pin(async {
            Ok(vec![
                Contact {
                    id: user_id!("a@prose.org"),
                    name: None,
                    group: Group::Favorite,
                },
                Contact {
                    id: user_id!("b@prose.org"),
                    name: Some("Contact B".to_string()),
                    group: Group::Team,
                },
                Contact {
                    id: user_id!("john.doe@prose.org"),
                    name: None,
                    group: Group::Team,
                },
            ])
        })
    });

    deps.user_info_repo
        .expect_get_user_info()
        .times(3)
        .returning(|jid| {
            let info = match &jid {
                _ if jid == &user_id!("a@prose.org") => Some(UserInfo {
                    avatar: None,
                    activity: None,
                    availability: Availability::Available,
                }),
                _ if jid == &user_id!("b@prose.org") => Some(UserInfo {
                    avatar: None,
                    activity: None,
                    availability: Availability::Available,
                }),
                _ if jid == &user_id!("john.doe@prose.org") => None,
                _ => unreachable!(),
            };

            Box::pin(async move { Ok(info) })
        });

    deps.user_profile_repo
        .expect_get()
        .times(3)
        .returning(|jid| {
            let mut profile = UserProfile::default();

            match &jid {
                _ if jid == &user_id!("a@prose.org") => {
                    profile.first_name = Some("First".to_string());
                    profile.last_name = Some("Last".to_string());
                }
                _ if jid == &user_id!("b@prose.org") => {
                    profile.nickname = Some("Nickname".to_string());
                }
                _ if jid == &user_id!("john.doe@prose.org") => (),
                _ => unreachable!(),
            };

            Box::pin(async move { Ok(Some(profile)) })
        });

    let service = ContactsService::from(&deps.into_deps());

    let contacts = service.load_contacts().await?;
    assert_eq!(
        contacts,
        vec![
            ContactDTO {
                id: user_id!("a@prose.org"),
                name: "First Last".to_string(),
                availability: Availability::Available,
                activity: None,
                group: Group::Favorite,
            },
            ContactDTO {
                id: user_id!("b@prose.org"),
                name: "Nickname".to_string(),
                availability: Availability::Available,
                activity: None,
                group: Group::Team,
            },
            ContactDTO {
                id: user_id!("john.doe@prose.org"),
                name: "John Doe".to_string(),
                availability: Availability::Unavailable,
                activity: None,
                group: Group::Team,
            }
        ]
    );

    Ok(())
}
