// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use xmpp_parsers::Element;

// -- Structures --

pub enum ProseBrokerIngressEventMessaging {
    MessageReceive(Element),
    MessageCorrect(Element),
    MessageHistory(Element),
    ComposeState(Element),
}
