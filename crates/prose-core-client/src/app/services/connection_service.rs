// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use tracing::error;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::{ConnectionError, IDProvider};

use crate::app::deps::{
    DynAccountSettingsRepository, DynAppContext, DynClientEventDispatcher, DynConnectionService,
    DynIDProvider, DynUserAccountService,
};
use crate::client_event::ConnectionEvent;
use crate::domain::connection::models::ConnectionProperties;
use crate::domain::shared::models::Availability;
use crate::ClientEvent;

#[derive(InjectDependencies)]
pub struct ConnectionService {
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
}

impl ConnectionService {
    pub async fn connect(
        &self,
        jid: &BareJid,
        password: impl AsRef<str>,
    ) -> Result<(), ConnectionError> {
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
        let availability = settings.availability.unwrap_or(Availability::Available);

        let full_jid = jid
            .with_resource_str(&resource)
            .expect("Failed to build FullJid with generated ID as resource.");

        self.ctx.set_connection_properties(ConnectionProperties {
            connected_jid: full_jid.clone(),
            server_features: Default::default(),
        });

        let connection_result = self
            .connection_service
            .connect(&full_jid, password.as_ref())
            .await;
        match connection_result {
            Ok(_) => (),
            Err(err) => {
                self.ctx.reset_connection_properties();
                return Err(err);
            }
        }

        self.user_account_service
            .set_availability(&self.ctx.capabilities, &availability)
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
        self.ctx.set_connection_properties(ConnectionProperties {
            connected_jid: full_jid.clone(),
            server_features,
        });

        self.account_settings_repo
            .update(
                jid,
                Box::new(move |settings| {
                    settings.resource = Some(resource);
                    settings.availability = Some(availability);
                }),
            )
            .await
            .map_err(|err| ConnectionError::Generic {
                msg: err.to_string(),
            })?;

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::ConnectionStatusChanged {
                event: ConnectionEvent::Connect,
            });

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
