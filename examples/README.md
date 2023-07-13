The examples in this directory require authentication with a XMPP server. To provide your credentials you can either copy the .env.example to .env and insert your credentials or provide them via the command line, e. g. `run --package xmpp-client account@your-server.com your-password`.

By default logging is done via the tracing-oslog crate. You can see the output on macOS in the Console.app. To only see relevant output set a filter to `subsystem:org.prose`. 