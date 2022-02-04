// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use xmpp_parsers;
use tokio_xmpp;

// -- Structures --

pub enum ProseClientOrigin {
    TestsCLI,
    ProseAppMacOS,
    ProseAppIOS,
    ProseAppAndroid,
    ProseAppWindows,
    ProseAppLinux,
    ProseAppWeb,
}

pub struct ProseClient {
    pub jid: BareJid,
    pub password: String,
    pub origin: ProseClientOrigin
}
