// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::util::form_config::{FormValue, Value};
use crate::util::{form_config, FormConfig};
use jid::BareJid;
use prose_xmpp::ns;
use prose_xmpp::stanza::muc::ns::roomconfig as cfg;
use xmpp_parsers::data_forms::{DataForm, DataFormType};

#[derive(Debug, Clone)]
pub(crate) struct RoomConfig {
    config: FormConfig,
}

impl RoomConfig {
    pub fn group(name: impl AsRef<str>, participants: &[BareJid]) -> Self {
        let history_len = Value::TextSingle(u32::MAX.to_string());
        RoomConfig {
            config: FormConfig::new([
                FormValue::optional(cfg::ALLOW_INVITES, Value::Boolean(false)),
                FormValue::optional(cfg::ALLOW_MEMBER_INVITES, Value::Boolean(false)),
                FormValue::optional(cfg::ALLOW_PM, Value::TextSingle("none".to_string())),
                FormValue::optional(cfg::CHANGE_SUBJECT, Value::Boolean(true)),
                FormValue::optional(cfg::DEFAULT_HISTORY_MESSAGES, history_len.clone()),
                FormValue::optional(cfg::ENABLE_LOGGING, Value::Boolean(false)),
                FormValue::optional(cfg::LANG, Value::TextSingle("en".to_string())),
                FormValue::optional(cfg::MAX_HISTORY_FETCH, history_len),
                FormValue::optional(cfg::MAX_USERS, Value::TextSingle("10".to_string())),
                FormValue::optional(cfg::MODERATED_ROOM, Value::Boolean(false)),
                FormValue::optional(cfg::PASSWORD_PROTECTED_ROOM, Value::Boolean(false)),
                FormValue::optional(
                    cfg::PRESENCE_BROADCAST,
                    Value::ListMulti(vec![
                        "moderator".to_string(),
                        "participant".to_string(),
                        "visitor".to_string(),
                    ]),
                ),
                FormValue::optional(cfg::PUBSUB, Value::None),
                FormValue::optional(
                    cfg::ROOM_ADMINS,
                    Value::JidMulti(participants.iter().map(|jid| jid.clone().into()).collect()),
                ),
                FormValue::optional(cfg::ROOM_DESC, Value::None),
                FormValue::optional(cfg::ROOM_NAME, Value::TextSingle(name.as_ref().to_string())),
                FormValue::optional(cfg::ROOM_OWNERS, Value::None),
                FormValue::optional(cfg::ROOM_SECRET, Value::None),
                FormValue::optional(cfg::GET_MEMBER_LIST, Value::Boolean(true)),
                FormValue::optional(cfg::WHOIS, Value::ListSingle("anyone".to_string())),
                FormValue::optional(cfg::MEMBERS_ONLY, Value::Boolean(true)),
                FormValue::optional(cfg::PERSISTENT_ROOM, Value::Boolean(true)),
                FormValue::optional(cfg::PUBLIC_ROOM, Value::Boolean(false)),
            ]),
        }
    }

    pub fn private_channel(name: impl AsRef<str>) -> Self {
        let history_len = Value::TextSingle(u32::MAX.to_string());
        RoomConfig {
            config: FormConfig::new([
                FormValue::optional(cfg::ALLOW_INVITES, Value::Boolean(true)),
                FormValue::optional(cfg::ALLOW_MEMBER_INVITES, Value::Boolean(true)),
                FormValue::optional(cfg::ALLOW_PM, Value::TextSingle("none".to_string())),
                FormValue::optional(cfg::CHANGE_SUBJECT, Value::Boolean(true)),
                FormValue::optional(cfg::DEFAULT_HISTORY_MESSAGES, history_len.clone()),
                FormValue::optional(cfg::ENABLE_LOGGING, Value::Boolean(false)),
                FormValue::optional(cfg::LANG, Value::TextSingle("en".to_string())),
                FormValue::optional(cfg::MAX_HISTORY_FETCH, history_len),
                FormValue::optional(cfg::MAX_USERS, Value::TextSingle("100".to_string())),
                FormValue::optional(cfg::MODERATED_ROOM, Value::Boolean(false)),
                FormValue::optional(cfg::PASSWORD_PROTECTED_ROOM, Value::Boolean(false)),
                FormValue::optional(
                    cfg::PRESENCE_BROADCAST,
                    Value::ListMulti(vec![
                        "moderator".to_string(),
                        "participant".to_string(),
                        "visitor".to_string(),
                    ]),
                ),
                FormValue::optional(cfg::PUBSUB, Value::None),
                FormValue::optional(cfg::ROOM_ADMINS, Value::None),
                FormValue::optional(cfg::ROOM_DESC, Value::None),
                FormValue::optional(cfg::ROOM_NAME, Value::TextSingle(name.as_ref().to_string())),
                FormValue::optional(cfg::ROOM_OWNERS, Value::None),
                FormValue::optional(cfg::ROOM_SECRET, Value::None),
                FormValue::optional(cfg::GET_MEMBER_LIST, Value::Boolean(true)),
                FormValue::optional(cfg::WHOIS, Value::ListSingle("anyone".to_string())),
                FormValue::optional(cfg::MEMBERS_ONLY, Value::Boolean(true)),
                FormValue::optional(cfg::PERSISTENT_ROOM, Value::Boolean(true)),
                FormValue::optional(cfg::PUBLIC_ROOM, Value::Boolean(false)),
            ]),
        }
    }

    pub fn public_channel(name: impl AsRef<str>) -> Self {
        let history_len = Value::TextSingle(u32::MAX.to_string());
        RoomConfig {
            config: FormConfig::new([
                FormValue::optional(cfg::ALLOW_INVITES, Value::Boolean(true)),
                FormValue::optional(cfg::ALLOW_MEMBER_INVITES, Value::Boolean(true)),
                FormValue::optional(cfg::ALLOW_PM, Value::TextSingle("none".to_string())),
                FormValue::optional(cfg::CHANGE_SUBJECT, Value::Boolean(false)),
                FormValue::optional(cfg::DEFAULT_HISTORY_MESSAGES, history_len.clone()),
                FormValue::optional(cfg::ENABLE_LOGGING, Value::Boolean(false)),
                FormValue::optional(cfg::LANG, Value::TextSingle("en".to_string())),
                FormValue::optional(cfg::MAX_HISTORY_FETCH, history_len),
                FormValue::optional(cfg::MAX_USERS, Value::TextSingle("100".to_string())),
                FormValue::optional(cfg::MODERATED_ROOM, Value::Boolean(false)),
                FormValue::optional(cfg::PASSWORD_PROTECTED_ROOM, Value::Boolean(false)),
                FormValue::optional(
                    cfg::PRESENCE_BROADCAST,
                    Value::ListMulti(vec![
                        "moderator".to_string(),
                        "participant".to_string(),
                        "visitor".to_string(),
                    ]),
                ),
                FormValue::optional(cfg::PUBSUB, Value::None),
                FormValue::optional(cfg::ROOM_ADMINS, Value::None),
                FormValue::optional(cfg::ROOM_DESC, Value::None),
                FormValue::optional(cfg::ROOM_NAME, Value::TextSingle(name.as_ref().to_string())),
                FormValue::optional(cfg::ROOM_OWNERS, Value::None),
                FormValue::optional(cfg::ROOM_SECRET, Value::None),
                FormValue::optional(cfg::WHOIS, Value::ListSingle("anyone".to_string())),
                FormValue::optional(cfg::GET_MEMBER_LIST, Value::Boolean(true)),
                FormValue::optional(cfg::MEMBERS_ONLY, Value::Boolean(false)),
                FormValue::optional(cfg::PERSISTENT_ROOM, Value::Boolean(true)),
                FormValue::optional(cfg::PUBLIC_ROOM, Value::Boolean(true)),
            ]),
        }
    }
}

impl RoomConfig {
    pub fn populate_form(&self, form: &DataForm) -> Result<DataForm, form_config::Error> {
        Ok(DataForm {
            type_: DataFormType::Submit,
            form_type: Some(ns::MUC_ROOMCONFIG.to_string()),
            title: None,
            instructions: None,
            fields: self.config.populate_form_fields(&form.fields)?,
        })
    }
}
