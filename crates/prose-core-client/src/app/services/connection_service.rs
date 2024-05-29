// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use secrecy::Secret;
use tracing::error;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::{ConnectionError, IDProvider, TimeProvider};

use crate::app::deps::{
    DynAccountSettingsRepository, DynAppContext, DynBlockListDomainService,
    DynClientEventDispatcher, DynConnectionService, DynContactListDomainService,
    DynEncryptionDomainService, DynIDProvider, DynOfflineMessagesRepository,
    DynServerEventHandlerQueue, DynTimeProvider, DynUserAccountService, DynUserProfileRepository,
};
use crate::app::event_handlers::ServerEvent;
use crate::client_event::ConnectionEvent;
use crate::domain::connection::models::ConnectionProperties;
use crate::domain::shared::models::ConnectionState;
use crate::dtos::UserId;
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
    user_profile_repo: DynUserProfileRepository,
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
        jid: &UserId,
        password: Secret<String>,
    ) -> Result<(), ConnectionError> {
        self.ctx.set_connection_state(ConnectionState::Connecting);
        self.offline_messages_repo.drain();

        let settings =
            self.account_settings_repo
                .get(jid)
                .await
                .map_err(|err| ConnectionError::Generic {
                    msg: err.to_string(),
                })?;
        let resource = settings
            .resource
            .unwrap_or_else(|| self.short_id_provider.new_id());
        let availability = settings.availability;

        let full_jid = jid
            .with_resource(&resource)
            .expect("Failed to build FullJid with generated ID as resource.");

        let mut connection_properties = ConnectionProperties {
            connected_jid: full_jid.clone(),
            server_features: Default::default(),
            connection_timestamp: DateTime::<Utc>::MIN_UTC,
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

        // https://xmpp.org/rfcs/rfc6121.html#roster-login
        if let Err(error) = self.contact_list_domain_service.load_contacts().await {
            error!("Failed to load contact list. {}", error.to_string());
        }

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
                jid,
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

        self.user_profile_repo.reset_after_reconnect().await;

        self.ctx.set_connection_state(ConnectionState::Connected);

        for event in self.offline_messages_repo.drain() {
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
        self.ctx.connection_properties.write().take();
    }
}

#[cfg(feature = "debug")]
impl ConnectionService {
    pub async fn send_raw_stanza(&self, stanza: minidom::Element) -> anyhow::Result<()> {
        self.connection_service.send_raw_stanza(stanza).await
    }
}
