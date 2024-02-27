// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use xmpp_parsers::data_forms::{DataForm, DataFormType};

use prose_xmpp::mods::muc::RoomConfigResponse;
use prose_xmpp::stanza::muc;
use prose_xmpp::{mods, ns};

use crate::domain::rooms::services::RoomAttributesService;
use crate::domain::shared::models::MucId;
use crate::infra::xmpp::XMPPClient;
use crate::util::form_config::{FormValue, Value};
use crate::util::FormConfig;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl RoomAttributesService for XMPPClient {
    async fn set_topic(&self, room_id: &MucId, subject: Option<&str>) -> Result<()> {
        let muc = self.client.get_mod::<mods::MUC>();
        muc.set_room_subject(room_id.as_ref(), subject).await
    }

    async fn set_name(&self, room_id: &MucId, name: &str) -> Result<()> {
        let config = FormConfig::new([FormValue::required(
            muc::ns::roomconfig::ROOM_NAME,
            Value::TextSingle(name.to_string()),
        )]);

        let muc = self.client.get_mod::<mods::MUC>();
        muc.configure_room(
            room_id.as_ref(),
            Box::new(|form: DataForm| {
                Box::pin(async move {
                    Ok(RoomConfigResponse::Submit(DataForm {
                        type_: DataFormType::Submit,
                        form_type: Some(ns::MUC_ROOMCONFIG.to_string()),
                        title: None,
                        instructions: None,
                        fields: config.populate_form_fields(&form.fields)?,
                    }))
                })
            }),
        )
        .await?;

        Ok(())
    }
}
