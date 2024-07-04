// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use pretty_assertions::assert_eq;

use prose_core_client::app::dtos::Contact as ContactDTO;
use prose_core_client::app::services::ContactListService;
use prose_core_client::domain::contacts::models::{Contact, PresenceSubscription};
use prose_core_client::domain::shared::models::{Availability, UserId};
use prose_core_client::domain::user_info::models::{ProfileName, UserInfo, UserName};
use prose_core_client::dtos::Group;
use prose_core_client::test::MockAppDependencies;
use prose_core_client::user_id;

#[tokio::test]
async fn test_assembles_contact_dto() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    deps.contact_list_domain_service
        .expect_load_contacts()
        .times(1)
        .returning(|| {
            Box::pin(async {
                Ok(vec![
                    Contact {
                        id: user_id!("a@prose.org"),
                        name: None,
                        presence_subscription: PresenceSubscription::Mutual,
                    },
                    Contact {
                        id: user_id!("b@prose.org"),
                        name: None,
                        presence_subscription: PresenceSubscription::WeFollow,
                    },
                    Contact {
                        id: user_id!("john.doe@prose.org"),
                        name: None,
                        presence_subscription: PresenceSubscription::TheyFollow,
                    },
                ])
            })
        });

    deps.user_info_domain_service
        .expect_get_user_info()
        .times(3)
        .returning(|jid, _| {
            let info = match &jid {
                _ if jid == &user_id!("a@prose.org") => Some(UserInfo {
                    name: UserName {
                        roster: None,
                        nickname: None,
                        presence: None,
                        vcard: Some(ProfileName {
                            first_name: Some("First".to_string()),
                            last_name: Some("Last".to_string()),
                            nickname: None,
                        }),
                    },
                    availability: Availability::Available,
                    ..Default::default()
                }),
                _ if jid == &user_id!("b@prose.org") => Some(UserInfo {
                    name: UserName {
                        nickname: Some("Nickname".to_string()),
                        ..Default::default()
                    },
                    availability: Availability::Available,
                    ..Default::default()
                }),
                _ if jid == &user_id!("john.doe@prose.org") => None,
                _ => unreachable!(),
            };

            Box::pin(async move { Ok(info) })
        });

    let service = ContactListService::from(&deps.into_deps());

    let contacts = service.load_contacts().await?;
    assert_eq!(
        vec![
            ContactDTO {
                id: user_id!("a@prose.org"),
                name: "First Last".to_string(),
                availability: Availability::Available,
                status: None,
                group: Group::Team,
                presence_subscription: PresenceSubscription::Mutual,
            },
            ContactDTO {
                id: user_id!("b@prose.org"),
                name: "Nickname".to_string(),
                availability: Availability::Available,
                status: None,
                group: Group::Team,
                presence_subscription: PresenceSubscription::WeFollow,
            },
            ContactDTO {
                id: user_id!("john.doe@prose.org"),
                name: "John Doe".to_string(),
                availability: Availability::Unavailable,
                status: None,
                group: Group::Team,
                presence_subscription: PresenceSubscription::TheyFollow,
            }
        ],
        contacts,
    );

    Ok(())
}
