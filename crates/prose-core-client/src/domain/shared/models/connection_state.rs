// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}
