// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::{Arc, OnceLock};

use anyhow::Result;
use mockall::predicate;
use secrecy::{ExposeSecret, SecretString};

use prose_core_client::app::deps::DynAppContext;
use prose_core_client::app::services::ConnectionService;
use prose_core_client::domain::connection::models::ServerFeatures;
use prose_core_client::domain::settings::models::AccountSettings;
use prose_core_client::domain::shared::models::{AccountId, Availability, UserId, UserResourceId};
use prose_core_client::test::MockAppDependencies;
use prose_core_client::{account_id, user_id, user_resource_id, ClientEvent, ConnectionEvent};
use prose_xmpp::test::ConstantIDProvider;
use prose_xmpp::{bare, ConnectionError};

#[tokio::test]
async fn test_starts_available_and_generates_resource() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.offline_message_repo
        .expect_drain()
        .times(2)
        .returning(|| vec![]);

    deps.encryption_domain_service
        .expect_initialize()
        .once()
        .return_once(|| Box::pin(async { Ok(()) }));

    deps.user_info_domain_service
        .expect_reset_before_reconnect()
        .once()
        .return_once(|| Box::pin(async { Ok(()) }));
    deps.user_info_domain_service
        .expect_handle_contacts_changed()
        .once()
        .return_once(|_| Box::pin(async { Ok(()) }));
    deps.contact_list_domain_service
        .expect_reset_before_reconnect()
        .once()
        .return_once(|| Box::pin(async { Ok(()) }));
    deps.block_list_domain_service
        .expect_reset_before_reconnect()
        .once()
        .return_once(|| Box::pin(async { Ok(()) }));
    deps.encryption_domain_service
        .expect_reset_before_reconnect()
        .once()
        .return_once(|| Box::pin(async { Ok(()) }));

    deps.short_id_provider = Arc::new(ConstantIDProvider::new("resource-id"));

    deps.account_settings_repo
        .expect_get()
        .once()
        .return_once(|_| Box::pin(async { Ok(Default::default()) }));
    deps.connection_service
        .expect_connect()
        .once()
        .with(
            predicate::eq(user_resource_id!("jane.doe@prose.org/resource-id")),
            predicate::function(|pw: &SecretString| pw.expose_secret() == "my-password"),
        )
        .return_once(|_, _| Box::pin(async { Ok(Default::default()) }));
    deps.contact_list_domain_service
        .expect_load_contacts()
        .once()
        .return_once(|| Box::pin(async { Ok(vec![]) }));
    deps.connection_service
        .expect_set_message_carbons_enabled()
        .once()
        .return_once(|_| Box::pin(async { Ok(()) }));
    deps.user_account_service
        .expect_set_availability()
        .once()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(Availability::Available),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(Default::default()) }));
    deps.connection_service
        .expect_load_server_features()
        .once()
        .return_once(|| {
            Box::pin(async {
                let mut features = ServerFeatures::default();
                features.muc_service = Some(bare!("muc@prose.org"));
                Ok(features)
            })
        });
    deps.account_settings_repo
        .expect_update()
        .once()
        .with(
            predicate::eq(account_id!("jane.doe@prose.org")),
            predicate::always(),
        )
        .return_once(|_, f| {
            Box::pin(async {
                let mut settings = AccountSettings::default();
                f(&mut settings);
                assert_eq!(settings.availability, Availability::Available);
                assert_eq!(settings.resource, Some("resource-id".to_string()));
                Ok(())
            })
        });
    deps.block_list_domain_service
        .expect_load_block_list()
        .once()
        .return_once(|| Box::pin(async { Ok(vec![]) }));
    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::ConnectionStatusChanged {
            event: ConnectionEvent::Connect,
        }))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::AccountInfoChanged))
        .return_once(|_| ());

    let deps = deps.into_deps();

    let service = ConnectionService::from(&deps);

    *deps.ctx.connection_properties.write() = None;
    assert!(deps.ctx.connected_id().is_err());
    assert!(deps.ctx.muc_service().is_err());

    service
        .connect(&user_id!("jane.doe@prose.org"), "my-password".into())
        .await?;

    assert_eq!(
        deps.ctx.connected_id()?,
        user_resource_id!("jane.doe@prose.org/resource-id")
    );
    assert_eq!(deps.ctx.muc_service()?, bare!("muc@prose.org"));

    Ok(())
}

#[tokio::test]
async fn test_restores_availability_and_resource() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.offline_message_repo
        .expect_drain()
        .times(2)
        .returning(|| vec![]);

    deps.encryption_domain_service
        .expect_initialize()
        .once()
        .return_once(|| Box::pin(async { Ok(()) }));

    deps.user_info_domain_service
        .expect_reset_before_reconnect()
        .once()
        .return_once(|| Box::pin(async { Ok(()) }));
    deps.user_info_domain_service
        .expect_handle_contacts_changed()
        .once()
        .return_once(|_| Box::pin(async { Ok(()) }));
    deps.contact_list_domain_service
        .expect_reset_before_reconnect()
        .once()
        .return_once(|| Box::pin(async { Ok(()) }));
    deps.block_list_domain_service
        .expect_reset_before_reconnect()
        .once()
        .return_once(|| Box::pin(async { Ok(()) }));
    deps.encryption_domain_service
        .expect_reset_before_reconnect()
        .once()
        .return_once(|| Box::pin(async { Ok(()) }));

    deps.account_settings_repo
        .expect_get()
        .once()
        .return_once(|_| {
            Box::pin(async {
                let mut account_settings = AccountSettings::default();
                account_settings.availability = Availability::DoNotDisturb;
                account_settings.resource = Some("restored-res".to_string());
                Ok(account_settings)
            })
        });
    deps.connection_service
        .expect_connect()
        .once()
        .with(
            predicate::eq(user_resource_id!("jane.doe@prose.org/restored-res")),
            predicate::always(),
        )
        .return_once(|_, _| Box::pin(async { Ok(Default::default()) }));
    deps.contact_list_domain_service
        .expect_load_contacts()
        .once()
        .return_once(|| Box::pin(async { Ok(vec![]) }));
    deps.connection_service
        .expect_set_message_carbons_enabled()
        .once()
        .return_once(|_| Box::pin(async { Ok(()) }));
    deps.user_account_service
        .expect_set_availability()
        .once()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::eq(Availability::DoNotDisturb),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(Default::default()) }));
    deps.connection_service
        .expect_load_server_features()
        .once()
        .return_once(|| Box::pin(async { Ok(Default::default()) }));
    deps.account_settings_repo
        .expect_update()
        .once()
        .with(
            predicate::eq(account_id!("jane.doe@prose.org")),
            predicate::always(),
        )
        .return_once(|_, f| {
            Box::pin(async {
                let mut settings = AccountSettings::default();
                f(&mut settings);
                assert_eq!(settings.availability, Availability::DoNotDisturb);
                assert_eq!(settings.resource, Some("restored-res".to_string()));
                Ok(())
            })
        });
    deps.block_list_domain_service
        .expect_load_block_list()
        .once()
        .return_once(|| Box::pin(async { Ok(vec![]) }));

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::ConnectionStatusChanged {
            event: ConnectionEvent::Connect,
        }))
        .return_once(|_| ());

    deps.client_event_dispatcher
        .expect_dispatch_event()
        .once()
        .with(predicate::eq(ClientEvent::AccountInfoChanged))
        .return_once(|_| ());

    let deps = deps.into_deps();

    let service = ConnectionService::from(&deps);

    service
        .connect(&user_id!("jane.doe@prose.org"), "my-password".into())
        .await?;

    Ok(())
}

#[tokio::test]
/// Test that the ConnectionService sets the connected_jid on AppContext before it
/// starts connecting and clears it if the connection fails. It's important that the connected_jid
/// is set immediately so that when events come in while the connection is in progress,
/// event handlers already have access to the connected_jid.
async fn test_connection_failure() -> Result<()> {
    let mut deps = MockAppDependencies::default();
    let ctx = Arc::new(OnceLock::<DynAppContext>::new());

    deps.short_id_provider = Arc::new(ConstantIDProvider::new("resource-id"));

    deps.offline_message_repo
        .expect_drain()
        .once()
        .return_once(|| vec![]);

    deps.account_settings_repo
        .expect_get()
        .once()
        .return_once(|_| Box::pin(async { Ok(Default::default()) }));
    {
        let ctx = ctx.clone();
        deps.connection_service
            .expect_connect()
            .once()
            .return_once(move |_, _| {
                assert_eq!(
                    ctx.get().unwrap().connected_id().ok(),
                    Some(user_resource_id!("jane.doe@prose.org/resource-id"))
                );
                Box::pin(async {
                    Err(ConnectionError::Generic {
                        msg: "Failure".to_string(),
                    })
                })
            });
    }

    let deps = deps.into_deps();
    ctx.set(deps.ctx.clone()).map_err(|_| ()).unwrap();

    let service = ConnectionService::from(&deps);

    *deps.ctx.connection_properties.write() = None;
    assert!(deps.ctx.connected_id().is_err());
    assert!(deps.ctx.muc_service().is_err());

    assert!(service
        .connect(&user_id!("jane.doe@prose.org"), "my-password".into())
        .await
        .is_err());

    assert!(deps.ctx.connected_id().is_err());
    assert!(deps.ctx.muc_service().is_err());

    Ok(())
}
