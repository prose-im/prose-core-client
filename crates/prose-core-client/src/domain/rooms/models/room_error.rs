// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::stanza_error::DefinedCondition;

use prose_xmpp::RequestError;

use crate::domain::shared::models::RoomJid;

#[derive(thiserror::Error, Debug)]
pub enum RoomError {
    #[error("Room is already connected ({0}).")]
    RoomIsAlreadyConnected(RoomJid),
    #[error("No room exists with the specified JID.")]
    RoomNotFound,
    #[error("A public channel with the chosen name exists already.")]
    PublicChannelNameConflict,
    #[error("Group must have at least two participants.")]
    InvalidNumberOfParticipants,
    #[error(transparent)]
    RequestError(#[from] RequestError),
    #[error("{0}")]
    RoomValidationError(String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    JidError(#[from] jid::Error),
    #[error(transparent)]
    ParseError(#[from] prose_xmpp::ParseError),
}

impl RoomError {
    pub(crate) fn is_gone_err(&self) -> bool {
        let Self::RequestError(error) = &self else {
            return false;
        };
        error.defined_condition() == Some(DefinedCondition::Gone)
    }
}
