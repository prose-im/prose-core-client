use nanoid::nanoid;
use prose_xmpp::IDProvider;

#[derive(Default)]
pub struct NanoIDProvider {}

impl IDProvider for NanoIDProvider {
    fn new_id(&self) -> String {
        let chars = ('a'..='z')
            .chain('A'..='Z')
            .chain('0'..='9')
            .collect::<Vec<char>>();
        nanoid!(8, &chars)
    }
}
