// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use mockall::predicate;

use prose_core_client::domain::rooms::models::Room;
use prose_core_client::domain::settings::models::AccountSettings;
use prose_core_client::domain::shared::models::{OccupantId, RoomId, UserId};
use prose_core_client::dtos::Availability;
use prose_core_client::services::AccountService;
use prose_core_client::test::{mock_data, MockAppDependencies};
use prose_core_client::{occupant_id, room_id, user_id};

#[tokio::test]
async fn test_set_availability_updates_settings() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.user_account_service
        .expect_set_availability()
        .once()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(Availability::Away),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.connected_rooms_repo
        .expect_get_all()
        .once()
        .return_once(|| vec![]);

    deps.account_settings_repo
        .expect_update()
        .once()
        .with(
            predicate::eq(mock_data::account_jid().into_user_id()),
            predicate::always(),
        )
        .return_once(|_, f| {
            Box::pin(async {
                let mut settings = AccountSettings::default();
                f(&mut settings);
                assert_eq!(settings.availability, Availability::Away);
                Ok(())
            })
        });

    let service = AccountService::from(&deps.into_deps());
    service.set_availability(Availability::Away).await?;

    Ok(())
}

#[tokio::test]
async fn test_sends_availability_to_all_rooms() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.user_account_service
        .expect_set_availability()
        .once()
        .with(
            predicate::eq(None),
            predicate::always(),
            predicate::eq(Availability::Away),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.connected_rooms_repo
        .expect_get_all()
        .once()
        .return_once(|| {
            vec![
                Room::direct_message(user_id!("user@prose.org"), Availability::Available)
                    .with_user_nickname("nick"),
                Room::private_channel(room_id!("prc@conf.prose.org")).with_user_nickname("nick"),
                Room::public_channel(room_id!("pc@conf.prose.org")).with_user_nickname("nick"),
                Room::group(room_id!("group@conf.prose.org")).with_user_nickname("nick"),
            ]
        });

    deps.user_account_service
        .expect_set_availability()
        .once()
        .with(
            predicate::eq(Some(occupant_id!("prc@conf.prose.org/nick"))),
            predicate::always(),
            predicate::eq(Availability::Away),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));
    deps.user_account_service
        .expect_set_availability()
        .once()
        .with(
            predicate::eq(Some(occupant_id!("pc@conf.prose.org/nick"))),
            predicate::always(),
            predicate::eq(Availability::Away),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));
    deps.user_account_service
        .expect_set_availability()
        .once()
        .with(
            predicate::eq(Some(occupant_id!("group@conf.prose.org/nick"))),
            predicate::always(),
            predicate::eq(Availability::Away),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    deps.account_settings_repo
        .expect_update()
        .once()
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let service = AccountService::from(&deps.into_deps());
    service.set_availability(Availability::Away).await?;

    Ok(())
}
