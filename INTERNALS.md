# Internals — Prose Core Client

## The Big Picture

A Prose client can be built with the following code:

```rust
let client = ProseClientBuilder::new()
  .app(ProseClientOrigin::ProseAppMacOS)
  .build()?
  .bind()?;
```

Internally, this is a group of sub-clients, aka. accounts (a client has 1 or many accounts connected). In most cases, the Prose app will be bound to a single account anyway.

The client manager binds on a dedicated thread, and spawns its data store threads and client thread. It connects to each XMPP account stored in the database, and exposes convenience methods to send prepared payloads, or receive payloads. It gives away a `ProseClient`, which is then used to access and manage accounts (where 1 account = 1 sub-client).

So, stores and brokers get isolated per-account. The implementer application (eg. the macOS app) is given a list of clients on which it can bind and unbind its handlers at its whim. Thus, when switching to another Prose team in the UI, the user can simply use another client and rebind its event listeners on the ingress broker.
