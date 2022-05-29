// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Structures --

pub enum ProseBrokerIngressEventMessaging {
    MessageReceive(()),
    MessageCorrect(()),
    MessageHistory(()),
    ComposeState(()),
}
