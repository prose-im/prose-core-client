// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use minidom::Element;
use secrecy::Secret;
use tracing::{info, warn};

use prose_xmpp::{mods, ns, ConnectionError};

use crate::domain::connection::models::{HttpUploadService, ServerFeatures};
use crate::domain::connection::services::ConnectionService;
use crate::domain::shared::models::MamVersion;
use crate::dtos::UserResourceId;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ConnectionService for XMPPClient {
    async fn connect(
        &self,
        jid: &UserResourceId,
        password: Secret<String>,
    ) -> Result<(), ConnectionError> {
        self.client.connect(jid.as_ref(), password).await
    }

    async fn disconnect(&self) {
        self.client.disconnect()
    }

    async fn set_message_carbons_enabled(&self, is_enabled: bool) -> Result<()> {
        let chat = self.client.get_mod::<mods::Chat>();
        chat.set_message_carbons_enabled(is_enabled)?;
        Ok(())
    }

    async fn load_server_features(&self) -> Result<ServerFeatures> {
        let caps = self.client.get_mod::<mods::Caps>();
        let disco_items = caps.query_server_disco_items(None).await?;
        let mut server_features = ServerFeatures::default();

        for item in disco_items.items {
            info!("Loading features for {}…", item.jid);
            let info = match caps.query_disco_info(item.jid.clone(), None).await {
                Ok(info) => info,
                Err(error) => {
                    warn!(
                        "Failed to load server feature info for {}. {}",
                        item.jid,
                        error.to_string()
                    );
                    continue;
                }
            };

            let Some(identity) = info.identities.first() else {
                continue;
            };

            match identity.category.as_str() {
                "conference" if info.features.iter().find(|f| f.var == ns::MUC).is_some() => {
                    server_features.muc_service = Some(item.jid.into_bare())
                }
                "store"
                    if info
                        .features
                        .iter()
                        .find(|f| f.var == ns::HTTP_UPLOAD)
                        .is_some() =>
                {
                    let max_file_size = info
                        .extensions
                        .iter()
                        .find(|form| form.form_type.as_deref() == Some(ns::HTTP_UPLOAD))
                        .and_then(|form| {
                            form.fields
                                .iter()
                                .find(|field| field.var.as_deref() == Some("max-file-size"))
                        })
                        .and_then(|field| field.values.first())
                        .and_then(|value| value.parse::<u64>().ok());

                    server_features.http_upload_service = Some(HttpUploadService {
                        host: item.jid.into_bare(),
                        max_file_size: max_file_size.unwrap_or(u64::MAX),
                    });
                }
                _ => continue,
            }
        }

        info!("Loading server features…");
        let disco_info = caps
            .query_disco_info(
                self.connected_jid()
                    .ok_or(anyhow!("Not connected"))?
                    .into_bare(),
                None,
            )
            .await?;

        for feature in disco_info.features {
            match feature.var.as_ref() {
                ns::MAM1 => {
                    server_features.mam_version = Some(
                        server_features
                            .mam_version
                            .map_or(MamVersion::Mam1, |v| v.max(MamVersion::Mam1)),
                    )
                }
                ns::MAM2 => {
                    server_features.mam_version = Some(
                        server_features
                            .mam_version
                            .map_or(MamVersion::Mam2, |v| v.max(MamVersion::Mam2)),
                    )
                }
                ns::MAM2_EXTENDED => {
                    server_features.mam_version = Some(
                        server_features
                            .mam_version
                            .map_or(MamVersion::Mam2Extended, |v| {
                                v.max(MamVersion::Mam2Extended)
                            }),
                    )
                }
                ns::VCARD4 => {
                    server_features.vcard4 = true;
                }
                ns::AVATAR_PEP_VCARD_CONVERSION => {
                    server_features.avatar_pep_vcard_conversion = true;
                }
                _ => (),
            }
        }

        info!("Loading server time…");
        let profile = self.client.get_mod::<mods::Profile>();
        let t1 = Utc::now();
        let server_time = profile
            .load_server_time()
            .await
            .inspect_err(|err| warn!("Failed to load server time. {}", err.to_string()))
            .map(Some)
            .unwrap_or_default();
        let t2 = Utc::now();

        server_features.server_time_offset = server_time
            .map(|server_time| {
                let round_trip = t2.signed_duration_since(t1);
                let half_round_trip = round_trip / 2;
                let midpoint_time = t1 + half_round_trip;

                server_time.signed_duration_since(midpoint_time)
            })
            .unwrap_or_default();

        Ok(server_features)
    }

    async fn send_raw_stanza(&self, stanza: Element) -> Result<()> {
        Ok(self.client.send_raw_stanza(stanza)?)
    }
}
