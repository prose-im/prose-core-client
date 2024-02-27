use prose_core_client::domain::shared::models::MucId;
use prose_core_client::dtos::PublicRoomInfo;
use prose_core_client::muc_id;
use prose_core_client::services::RoomsService;
use prose_core_client::test::MockAppDependencies;

#[tokio::test]
async fn test_find_public_channel_by_name() -> anyhow::Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.room_management_service
        .expect_load_public_rooms()
        .returning(|_| {
            Box::pin(async {
                Ok(vec![
                    PublicRoomInfo {
                        id: muc_id!("dev-core@muc.prose.org").into(),
                        name: Some("Dev-Core".to_string()),
                    },
                    PublicRoomInfo {
                        id: muc_id!("dev-web@muc.prose.org").into(),
                        name: Some("dev-web".to_string()),
                    },
                ])
            })
        });

    let service = RoomsService::from(&deps.into_deps());

    assert_eq!(
        service.find_public_channel_by_name("dev-core").await?,
        Some(muc_id!("dev-core@muc.prose.org").into())
    );
    assert_eq!(
        service.find_public_channel_by_name("Dev-Web").await?,
        Some(muc_id!("dev-web@muc.prose.org").into())
    );
    assert_eq!(service.find_public_channel_by_name("dev-pod").await?, None);

    Ok(())
}
