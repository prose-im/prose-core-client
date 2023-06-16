use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use jid::FullJid;
use tracing::{error, info, warn};

use crate::connector::{Connection, ConnectionConfiguration, Connector, LibstropheConnector};
use crate::dependencies::{IDProvider, SystemTimeProvider, TimeProvider, UUIDProvider};
use crate::helpers::CompoundModule;
use crate::modules::{Context, Module, PendingRequest, RequestError, XMPPElement};
use crate::stanza::message::Kind;
use crate::stanza::pubsub::Event;
use crate::stanza::{iq, Message, Namespace, StanzaBase, IQ};
use crate::{modules, ConnectionError, ConnectionEvent, ConnectionHandler};

pub struct Client {
    connector: Box<dyn Connector>,
    connection_handler: Option<ConnectionHandler>,
    id_provider: Box<dyn IDProvider>,
    time_provider: Box<dyn TimeProvider>,
    pending_request_futures: Mutex<HashMap<iq::Id, PendingRequest>>,
    modules: CompoundModule,
}

impl Client {
    pub fn new() -> Self {
        Client {
            connector: Box::new(LibstropheConnector::default()),
            connection_handler: None,
            id_provider: Box::new(UUIDProvider::new()),
            time_provider: Box::new(SystemTimeProvider::new()),
            pending_request_futures: Mutex::new(HashMap::new()),
            modules: CompoundModule::new(),
        }
    }

    pub fn set_connector(mut self, connector: Box<dyn Connector>) -> Self {
        self.connector = connector;
        self
    }

    pub fn set_connection_handler(
        mut self,
        handler: impl FnMut(&dyn Connection, &ConnectionEvent) + Send + 'static,
    ) -> Self {
        self.connection_handler = Some(Box::new(handler));
        self
    }

    pub fn register_module(mut self, module: Arc<dyn Module + Send + Sync>) -> Self {
        self.modules.add_module(module);
        self
    }

    pub fn set_id_provider<P: IDProvider + 'static>(mut self, id_provider: P) -> Self {
        self.id_provider = Box::new(id_provider);
        self
    }

    pub fn set_time_provider<T: TimeProvider + 'static>(mut self, time_provider: T) -> Self {
        self.time_provider = Box::new(time_provider);
        self
    }
}

impl Client {
    pub async fn connect(
        self,
        jid: &FullJid,
        password: impl Into<String>,
    ) -> anyhow::Result<ConnectedClient, ConnectionError> {
        info!("Waiting for connection…");
        let now = Instant::now();

        let mut connection_handler = self.connection_handler.unwrap_or(Box::new(|_, _| {}));
        let modules = Arc::new(self.modules);
        let id_provider = Arc::new(self.id_provider);
        let time_provider = Arc::new(self.time_provider);
        let pending_request_futures = Arc::new(self.pending_request_futures);
        let conn_module = Arc::new(modules::Connection::new());

        let mut config = ConnectionConfiguration::new(jid.clone(), password);

        {
            let modules = modules.clone();
            let jid = jid.clone();
            let id_provider = id_provider.clone();
            let time_provider = time_provider.clone();
            let pending_request_futures = pending_request_futures.clone();

            config.connection_handler = Box::new(move |conn, event| {
                let result: anyhow::Result<()> = {
                    match event {
                        ConnectionEvent::Connect => {
                            info!("Received connection callback after {:.2?}", now.elapsed());

                            let ctx = Context::new(
                                &jid,
                                conn,
                                &**id_provider,
                                &**time_provider,
                                &pending_request_futures,
                            );
                            modules.handle_connect(&ctx)
                        }
                        ConnectionEvent::Disconnect { .. } => modules.handle_disconnect(),
                    }
                };

                connection_handler(conn, event);

                if let Err(error) = result {
                    error!("{:?}", error)
                }
            });
        }

        {
            let modules = modules.clone();
            let jid = jid.clone();
            let id_provider = id_provider.clone();
            let time_provider = time_provider.clone();
            let pending_request_futures = pending_request_futures.clone();

            config.stanza_handler = Box::new(move |conn, stanza| {
                let Some(name) = stanza.name() else {
                    return
                };

                let ctx = Context::new(
                    &jid,
                    conn,
                    &**id_provider,
                    &**time_provider,
                    &pending_request_futures,
                );

                let result: anyhow::Result<()> = {
                    match name {
                        "iq" => {
                            if let Err(err) =
                                modules.handle_element(&ctx, &XMPPElement::IQ(stanza.into()))
                            {
                                error!("{:?}", err);
                            }

                            let iq: IQ = stanza.into();

                            let Some(id) = iq.id() else {
                                return
                            };

                            let result = RequestError::try_from(&iq)
                                .ok()
                                .map(Result::Err)
                                .unwrap_or(Ok(iq.clone()));

                            if let Err(err) = ctx.handle_iq_result(&id, result) {
                                error!("{:?}", err);
                            }

                            Ok(())
                        }
                        "presence" => {
                            modules.handle_element(&ctx, &XMPPElement::Presence(stanza.into()))
                        }
                        "message" => {
                            let message: Message = stanza.into();

                            if message.kind() != Some(Kind::Headline) {
                                if let Err(err) = modules
                                    .handle_element(&ctx, &XMPPElement::Message(message.into()))
                                {
                                    error!("{:?}", err);
                                }
                                return;
                            }

                            let Some((from, event)) = message.from().zip(message.child_by_name_and_namespace("event", Namespace::PubSubEvent).map(Into::<Event>::into)) else {
                                warn!("Received unexpected headline message.");
                                return
                            };

                            let Some(first_child) = event.first_child() else {
                                warn!("Received unexpected empty pubsub event");
                              return
                            };

                            let Some(node) = first_child.attribute("node").and_then(|s| s.parse::<Namespace>().ok()) else {
                                warn!("Missing node attribute in pubsub event");
                                return
                            };

                            if let Err(err) =
                                modules.handle_pubsub_event(&ctx, &from, &node, &event)
                            {
                                error!("{:?}", err);
                            }

                            Ok(())
                        }
                        _ => modules.handle_element(&ctx, &XMPPElement::Other(stanza.into())),
                    }
                };

                if let Err(error) = result {
                    error!("{:?}", error)
                }
            });
        }

        {
            let conn_module = conn_module.clone();
            let jid = jid.clone();
            let id_provider = id_provider.clone();
            let time_provider = time_provider.clone();
            let pending_request_futures = pending_request_futures.clone();

            config.ping_handler = Box::new(move |conn| {
                let ctx = Context::new(
                    &jid,
                    conn,
                    &**id_provider,
                    &**time_provider,
                    &pending_request_futures,
                );

                if let Err(error) = conn_module.send_ping(&ctx) {
                    error!("{:?}", error)
                }

                true
            });
        }

        {
            let jid = jid.clone();
            let id_provider = id_provider.clone();
            let time_provider = time_provider.clone();
            let pending_request_futures = pending_request_futures.clone();

            config.timeout_handler = Box::new(move |conn| {
                let ctx = Context::new(
                    &jid,
                    conn,
                    &**id_provider,
                    &**time_provider,
                    &pending_request_futures,
                );
                if let Err(error) = ctx.purge_pending_futures() {
                    error!("{:?}", error)
                }
                true
            });
        }

        let connection = self.connector.connect(config).await?;
        info!("Connection established after {:.2?}", now.elapsed());

        Ok(ConnectedClient {
            jid: jid.clone(),
            connection,
            id_provider,
            time_provider,
            pending_request_futures,
        })
    }
}

pub struct ConnectedClient {
    pub jid: FullJid,
    connection: Box<dyn Connection>,
    id_provider: Arc<Box<dyn IDProvider>>,
    time_provider: Arc<Box<dyn TimeProvider>>,
    pending_request_futures: Arc<Mutex<HashMap<iq::Id, PendingRequest>>>,
}

impl ConnectedClient {
    pub fn context(&self) -> Context {
        let p = &**self.id_provider.as_ref();
        let t = &**self.time_provider.as_ref();
        Context::new(
            &self.jid,
            &*self.connection,
            p,
            t,
            &self.pending_request_futures,
        )
    }

    pub fn disconnect(self) {
        self.connection.disconnect();
    }
}
