The examples in this directory require authentication with a XMPP server. To provide your credentials you can either copy the .env.example to .env and insert your credentials or provide them via the command line, e. g. `cargo run --package xmpp-client 'account@your-server.com' 'your-password'`.

If you're trying to connect to a Prosody instance that uses self-signed certificates enable the feature `insecure-tcp` (e.g. `cargo run -p prose-core-client-cli --features insecure-tcp 'user@localhost' 'pw'`) and set `c2s_require_encryption = false` in your `prosody.cfg.lua`.

By default logging is done via the tracing-oslog crate. You can see the output on macOS in the Console.app. To only see relevant output set a filter to `subsystem:org.prose`.
