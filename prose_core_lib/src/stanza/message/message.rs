use crate::helpers::id_string_macro::id_string;
use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::stanza::message::fallback::Fallback;
use crate::stanza::message::{chat_marker, ChatMarker, ChatState, Kind, MessageFastening};
use crate::stanza::{Delay, Namespace};

id_string!(Id);
id_string!(Emoji);
id_string!(StanzaId);

#[derive(Clone)]
pub struct Message<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Message<'a> {
    pub fn new() -> Self {
        return Message {
            stanza: libstrophe::Stanza::new_message(None, None, None).into(),
        };
    }

    /// Generates a new message delivery receipt for the message with id `message_id` in
    /// accordance to XEP-0184.
    pub fn new_receipt(message_id: Id) -> Self {
        Message::new().add_child(
            Stanza::new("received")
                .set_namespace(Namespace::Receipts)
                .set_attribute("id", &message_id),
        )
    }
}

impl<'a> Message<'a> {
    pub fn id(&self) -> Option<Id> {
        self.stanza.get_attribute("id").map(Into::into)
    }

    pub fn set_id(mut self, id: Id) -> Self {
        self.stanza
            .to_mut()
            .set_id(id.as_ref())
            .expect("Failed to set id");
        self
    }

    /// XEP-0359: Unique and Stable Stanza IDs
    pub fn stanza_id(&self) -> Option<StanzaId> {
        self.child_by_name_and_namespace("stanza-id", Namespace::StanzaID)
            .and_then(|c| c.attribute("id").map(Into::into))
    }

    pub fn set_origin_id(self, id: StanzaId) -> Self {
        self.add_child(
            Stanza::new("origin-id")
                .set_namespace(Namespace::StanzaID)
                .set_attribute("id", id.into_inner()),
        )
    }

    pub fn kind(&self) -> Option<Kind> {
        self.stanza
            .get_attribute("type")
            .and_then(|s| s.parse::<Kind>().ok())
    }

    pub fn set_kind(self, kind: Kind) -> Self {
        self.set_attribute("type", kind.to_string())
    }

    pub fn body(&self) -> Option<String> {
        let Some(body) = self.child_by_name("body") else {
            return None;
        };
        // If we have a body node but no text, let's return an empty string.
        Some(body.text().unwrap_or(String::from("")))
    }

    pub fn set_body(self, text: impl AsRef<str>) -> Self {
        self.add_child(Stanza::new_text_node("body", text))
    }

    pub fn chat_state(&self) -> Option<ChatState> {
        self.child_by_namespace(Namespace::ChatStates)
            .and_then(|c| c.name().and_then(|n| n.parse::<ChatState>().ok()))
    }

    pub fn set_chat_state(self, state: ChatState) -> Self {
        self.add_child(Stanza::new(state.to_string()).set_namespace(Namespace::ChatStates))
    }

    pub fn replace(&self) -> Option<Id> {
        self.child_by_name_and_namespace("replace", Namespace::LastMessageCorrection)
            .and_then(|c| c.attribute("id").map(|a| a.into()))
    }

    pub fn set_replace(self, id: Id) -> Self {
        self.add_child(
            Stanza::new("replace")
                .set_namespace(Namespace::LastMessageCorrection)
                .set_attribute("id", &id),
        )
    }

    pub fn set_request_receipt(self) -> Self {
        self.add_child(Stanza::new("request").set_namespace(Namespace::Receipts))
    }

    pub fn has_receipt_request(&self) -> bool {
        self.child_by_name_and_namespace("request", Namespace::Receipts)
            .is_some()
    }

    pub fn message_reactions(&self) -> Option<(Id, Vec<Emoji>)> {
        let Some(stanza) = self.child_by_name_and_namespace("reactions", Namespace::Reactions) else {
            return None;
        };
        let Some(id) = stanza.attribute("id") else {
            return  None;
        };
        Some((
            id.into(),
            stanza
                .children()
                .filter_map(|child| {
                    if child.name() != Some("reaction") {
                        return None;
                    }
                    child.text().map(|t| t.into())
                })
                .collect(),
        ))
    }

    pub fn set_message_reactions(self, id: Id, reactions: impl IntoIterator<Item = Emoji>) -> Self {
        self.add_child(
            Stanza::new("reactions")
                .set_attribute("id", &id)
                .set_namespace(Namespace::Reactions)
                .add_children(
                    reactions
                        .into_iter()
                        .map(|reaction| Stanza::new_text_node("reaction", reaction)),
                ),
        )
    }

    pub fn fastening(&self) -> Option<MessageFastening> {
        self.child_by_name_and_namespace("apply-to", Namespace::Fasten)
            .map(Into::into)
    }

    pub fn set_fastening(self, fastening: MessageFastening) -> Self {
        self.add_child(fastening)
    }

    pub fn fallback(&self) -> Option<Fallback> {
        self.child_by_name_and_namespace("fallback", Namespace::Fallback)
            .map(Into::into)
    }

    pub fn set_fallback(self, fallback: Fallback) -> Self {
        self.add_child(fallback)
    }

    pub fn delay(&self) -> Option<Delay> {
        self.stanza
            .get_child_by_name_and_ns("delay", Namespace::Delay.to_string())
            .map(Into::into)
    }

    pub fn set_markable(self) -> Self {
        self.add_child(Stanza::new("markable").set_namespace(Namespace::ChatMarkers))
    }

    pub fn add_marker(self, marker: ChatMarker) -> Self {
        self.add_child(marker)
    }

    pub fn received_marker(&self) -> Option<ChatMarker> {
        self.child_by_name_and_namespace(
            chat_marker::Kind::Received.to_string(),
            Namespace::ChatMarkers,
        )
        .map(Into::into)
    }

    pub fn displayed_marker(&self) -> Option<ChatMarker> {
        self.child_by_name_and_namespace(
            chat_marker::Kind::Displayed.to_string(),
            Namespace::ChatMarkers,
        )
        .map(Into::into)
    }

    pub fn acknowledged_marker(&self) -> Option<ChatMarker> {
        self.child_by_name_and_namespace(
            chat_marker::Kind::Acknowledged.to_string(),
            Namespace::ChatMarkers,
        )
        .map(Into::into)
    }
}

stanza_base!(Message);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_chat_state() -> anyhow::Result<()> {
        let message = r#"
      <message from="valerian@prose.org/mobile" to="marc@prose.org/home" id="purplecf8f33c0" type="chat">
        <body>How is it going?</body>
        <active xmlns="http://jabber.org/protocol/chatstates"/>
      </message>
      "#;

        let stanza = Message::from_str(message).unwrap();

        // assert_eq!(stanza.chat_state(), Some(ChatState::Active));
        assert_eq!(stanza.body().as_deref(), Some("How is it going?"));

        Ok(())
    }

    #[test]
    fn test_get_message_reactions() {
        let message = r#"
      <message from="a@prose.org" to="b@prose.org" id="id1" type="chat">
        <reactions id="id2" xmlns='urn:xmpp:reactions:0'>
            <reaction>👋</reaction>
            <reaction>🐢</reaction>
        </reactions>
      </message>
      "#;

        let stanza = Message::from_str(message).unwrap();
        assert_eq!(
            stanza.message_reactions(),
            Some(("id2".into(), vec!["👋".into(), "🐢".into()]))
        );
    }

    #[test]
    fn test_set_message_reactions() {
        let stanza =
            Message::new().set_message_reactions("id2".into(), vec!["👋".into(), "🐢".into()]);
        let message = r#"<message><reactions id="id2" xmlns="urn:xmpp:reactions:0"><reaction>👋</reaction><reaction>🐢</reaction></reactions></message>"#;
        assert_eq!(stanza.to_string(), message);
    }
}
