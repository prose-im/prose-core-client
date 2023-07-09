// TODO!

// use std::time::Duration;
//
// use crate::helpers::RequestError;
// use tracing::info;
// use xmpp_parsers::iq::Iq;
// use xmpp_parsers::ping::Ping;
//
// use crate::modules::{Context, Module};
//
// pub(crate) struct Connection {}
//
// impl Connection {
//     pub(crate) fn new() -> Self {
//         Connection {}
//     }
// }
//
// impl Module for Connection {}
//
// impl Connection {
//     pub fn send_ping(&self, ctx: &Context) -> Result<()> {
//         ctx.send_iq_with_timeout_cb(
//             Iq::from_get(ctx.generate_id(), Ping).with_from(ctx.jid.clone().into()),
//             Duration::from_secs(5),
//             |ctx, result| {
//                 if let Err(RequestError::TimedOut) = result {
//                     info!("Ping timed out. Disconnecting…");
//                     ctx.disconnect();
//                 }
//             },
//         )
//     }
// }
