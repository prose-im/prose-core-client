use xmpp_parsers::data_forms::{DataForm, DataFormType, Field, FieldType};
use xmpp_parsers::pubsub::pubsub;

pub trait PublishOptionsExt {
    fn for_private_data(additional_fields: impl IntoIterator<Item = Field>) -> Self;
}

impl PublishOptionsExt for pubsub::PublishOptions {
    // XEP-0223: Persistent Storage of Private Data via PubSub
    // https://xmpp.org/extensions/xep-0223.html#approach
    fn for_private_data(additional_fields: impl IntoIterator<Item = Field>) -> Self {
        pubsub::PublishOptions {
            form: Some(DataForm {
                type_: DataFormType::Submit,
                form_type: Some(String::from(
                    "http://jabber.org/protocol/pubsub#publish-options",
                )),
                title: None,
                instructions: None,
                fields: [
                    Field::new("pubsub#persist_items", FieldType::Boolean).with_value("true"),
                    Field::new("pubsub#access_model", FieldType::TextSingle)
                        .with_value("whitelist"),
                ]
                .into_iter()
                .chain(additional_fields.into_iter())
                .collect(),
            }),
        }
    }
}
