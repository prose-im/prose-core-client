use xmpp_parsers::data_forms::{DataForm, DataFormType, Field, FieldType};
use xmpp_parsers::pubsub::pubsub;

pub trait PublishOptionsExt {
    fn for_private_data() -> Self;
}

impl PublishOptionsExt for pubsub::PublishOptions {
    // XEP-0223: Persistent Storage of Private Data via PubSub
    // https://xmpp.org/extensions/xep-0223.html#approach
    fn for_private_data() -> Self {
        pubsub::PublishOptions {
            form: Some(DataForm {
                type_: DataFormType::Submit,
                form_type: Some(String::from(
                    "http://jabber.org/protocol/pubsub#publish-options",
                )),
                title: None,
                instructions: None,
                fields: vec![
                    Field {
                        var: String::from("pubsub#persist_items"),
                        type_: FieldType::Boolean,
                        label: None,
                        required: false,
                        media: vec![],
                        options: vec![],
                        values: vec![String::from("true")],
                    },
                    Field {
                        var: String::from("pubsub#access_model"),
                        type_: FieldType::TextSingle,
                        label: None,
                        required: false,
                        media: vec![],
                        options: vec![],
                        values: vec![String::from("whitelist")],
                    },
                ],
            }),
        }
    }
}
