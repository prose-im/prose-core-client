use microtype::microtype;

microtype! {
    #[derive(Debug)]
    pub prose_core_domain::ChatState {
        ChatState
    }
}

impl From<ChatState> for prose_core_lib::stanza::message::ChatState {
    fn from(value: ChatState) -> Self {
        use prose_core_lib::stanza::message::ChatState;

        match value.0 {
            prose_core_domain::ChatState::Active => ChatState::Active,
            prose_core_domain::ChatState::Composing => ChatState::Composing,
            prose_core_domain::ChatState::Gone => ChatState::Gone,
            prose_core_domain::ChatState::Inactive => ChatState::Inactive,
            prose_core_domain::ChatState::Paused => ChatState::Paused,
        }
    }
}
