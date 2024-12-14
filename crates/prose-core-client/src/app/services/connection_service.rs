// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use secrecy::SecretString;
use tracing::{error, info, warn};

use prose_proc_macros::InjectDependencies;
use prose_xmpp::{ConnectionError, IDProvider, TimeProvider};

use crate::app::deps::{
    DynAccountSettingsRepository, DynAppContext, DynBlockListDomainService,
    DynClientEventDispatcher, DynConnectionService, DynContactListDomainService,
    DynEncryptionDomainService, DynIDProvider, DynOfflineMessagesRepository,
    DynServerEventHandlerQueue, DynSidebarDomainService, DynTimeProvider, DynUserAccountService,
    DynUserInfoDomainService,
};
use crate::app::event_handlers::ServerEvent;
use crate::client_event::ConnectionEvent;
use crate::domain::connection::models::ConnectionProperties;
use crate::domain::shared::models::{AccountId, ConnectionState};
use crate::dtos::{DecryptionContext, UserId};
use crate::ClientEvent;

#[derive(InjectDependencies)]
pub struct ConnectionService {
    #[inject]
    block_list_domain_service: DynBlockListDomainService,
    #[inject]
    contact_list_domain_service: DynContactListDomainService,
    #[inject]
    ctx: DynAppContext,
    #[inject]
    connection_service: DynConnectionService,
    #[inject]
    account_settings_repo: DynAccountSettingsRepository,
    #[inject]
    user_account_service: DynUserAccountService,
    #[inject]
    short_id_provider: DynIDProvider,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
    #[inject]
    encryption_domain_service: DynEncryptionDomainService,
    #[inject]
    user_info_domain_service: DynUserInfoDomainService,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
    #[inject]
    time_provider: DynTimeProvider,
    #[inject]
    offline_messages_repo: DynOfflineMessagesRepository,
    #[inject]
    server_event_handler_queue: DynServerEventHandlerQueue,
}

impl ConnectionService {
    pub async fn connect(
        &self,
        user_id: &UserId,
        password: SecretString,
    ) -> Result<(), ConnectionError> {
        self.ctx.set_connection_state(ConnectionState::Connecting);
        self.offline_messages_repo.drain();

        let account = AccountId::from(user_id.clone().into_inner());

        let settings = self
            .account_settings_repo
            .get(&account)
            .await
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;
        let resource = settings
            .resource
            .unwrap_or_else(|| self.short_id_provider.new_id());
        let availability = settings.availability;

        let full_jid = user_id
            .with_resource(&resource)
            .expect("Failed to build FullJid with generated ID as resource.");

        let mut connection_properties = ConnectionProperties {
            connected_jid: full_jid.clone(),
            server_features: Default::default(),
            rooms_caught_up: false,
            connection_timestamp: DateTime::<Utc>::MIN_UTC,
            decryption_context: Some(DecryptionContext::default()),
        };

        self.ctx
            .set_connection_properties(connection_properties.clone());

        let connection_result = self.connection_service.connect(&full_jid, password).await;
        match connection_result {
            Ok(_) => (),
            Err(err) => {
                self.ctx.reset_connection_properties();
                return Err(err);
            }
        }

        connection_properties.connection_timestamp = self.time_provider.now();
        self.ctx
            .set_connection_properties(connection_properties.clone());

        self.reset_services_before_reconnect().await;

        // https://xmpp.org/rfcs/rfc6121.html#roster-login
        if let Ok(contacts) = self
            .contact_list_domain_service
            .load_contacts()
            .await
            .inspect_err(|error| error!("Failed to load contact list. {error}"))
        {
            _ = self
                .user_info_domain_service
                .handle_contacts_changed(contacts)
                .await
                .inspect_err(|error| error!("Failed to handle changed contacts. {error}"));
        };

        self.user_account_service
            .set_availability(None, &self.ctx.capabilities, availability)
            .await
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        if let Err(err) = self
            .connection_service
            .set_message_carbons_enabled(true)
            .await
        {
            error!(
                "Failed to enable message carbons. Reason: {}",
                err.to_string()
            );
        }

        let server_features = self
            .connection_service
            .load_server_features()
            .await
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        connection_properties.server_features = server_features;
        self.ctx
            .set_connection_properties(connection_properties.clone());

        self.account_settings_repo
            .update(
                &account,
                Box::new(move |settings| {
                    settings.resource = Some(resource);
                    settings.availability = availability;
                }),
            )
            .await
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        if let Err(error) = self.block_list_domain_service.load_block_list().await {
            error!("Failed to load block list. {}", error.to_string());
        }

        self.encryption_domain_service
            .initialize()
            .await
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        self.ctx.set_connection_state(ConnectionState::Connected);

        let offline_message_events = self.offline_messages_repo.drain();
        info!(
            "Applying {} cached offline messages…",
            offline_message_events.len()
        );
        for event in offline_message_events {
            self.server_event_handler_queue
                .handle_server_event(ServerEvent::Message(event))
                .await;
        }

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::ConnectionStatusChanged {
                event: ConnectionEvent::Connect,
            });

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::AccountInfoChanged);

        Ok(())
    }

    pub async fn disconnect(&self) {
        self.connection_service.disconnect().await;
        self.ctx.set_connection_state(ConnectionState::Disconnected);
        _ = self.sidebar_domain_service.handle_disconnect().await;
        self.ctx.connection_properties.write().take();
    }
}

impl ConnectionService {
    async fn reset_services_before_reconnect(&self) {
        _ = self
            .user_info_domain_service
            .reset_before_reconnect()
            .await
            .inspect_err(|err| {
                warn!(
                    "Failed to reset UserProfileRepository after reconnect. {}",
                    err.to_string()
                )
            });
        _ = self
            .contact_list_domain_service
            .reset_before_reconnect()
            .await
            .inspect_err(|err| {
                warn!(
                    "Failed to reset ContactListDomainService after reconnect. {}",
                    err.to_string()
                )
            });
        _ = self
            .block_list_domain_service
            .reset_before_reconnect()
            .await
            .inspect_err(|err| {
                warn!(
                    "Failed to reset BlockListDomainService after reconnect. {}",
                    err.to_string()
                )
            });
        _ = self
            .encryption_domain_service
            .reset_before_reconnect()
            .await
            .inspect_err(|err| {
                warn!(
                    "Failed to reset EncryptionDomainService after reconnect. {}",
                    err.to_string()
                )
            });
    }
}
