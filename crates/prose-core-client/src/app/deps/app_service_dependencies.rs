// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_xmpp::{IDProvider, TimeProvider};
use std::sync::Arc;

use crate::app::event_handlers::ClientEventDispatcher;

pub type DynTimeProvider = Arc<dyn TimeProvider>;
pub type DynIDProvider = Arc<dyn IDProvider>;
pub type DynEventDispatcher = Arc<ClientEventDispatcher>;

pub struct AppServiceDependencies {
    pub time_provider: DynTimeProvider,
    pub id_provider: DynIDProvider,
    pub short_id_provider: DynIDProvider,
    pub event_dispatcher: DynEventDispatcher,
}
