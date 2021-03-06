# prose-core-client

[![Test and Build](https://github.com/prose-im/prose-core-client/actions/workflows/test.yml/badge.svg?branch=master)](https://github.com/prose-im/prose-core-client/actions/workflows/test.yml)

**Prose core XMPP client manager and protocols.**

Copyright 2022, Prose Foundation - Released under the [Mozilla Public License 2.0](./LICENSE.md).

_Tested at Rust version: `rustc 1.60.0 (7737e0b5c 2022-04-04)`_

## Architecture

The Prose core client uses the [libstrophe](http://strophe.im/libstrophe/) library to provide with low-level XMPP protocol and connection management. It is built in C, and wrapped by [a Rust library](https://github.com/twistedfall/libstrophe) as to expose native Rust bindings.

It builds up upon this base layer and provides a common and identical interface to the XMPP world to Prose apps. Useful functions this library provides, for instance, are models and store management. This minimizes code reuse or code adaptations in different programming languages (eg. redefining a similar data model in Swift and Java at once).

The client library is organized into parts responsible for specific tasks, namely:

* Overall client manager: `client`
* Sending payloads: `broker/egress`
* Receiving payloads: `broker/ingress`
* Persistence: `store`
* Protocol tools: `protocol`
* Data types: `types`

_👉 Note that as the state of XMPP in Rust is still pretty immature, we could not find any sufficiently-advanced pure-Rust XMPP client library. Therefore, we opted for the excellent C-based `libstrophe` library. However, depending on a non-Rust binding creates a lot of overhead for the developer, thus we plan to create a pure-Rust port of `libstrophe` in the near future. We aim for a similar API to `libstrophe`, as we like its simplicity for the developer and how lightweight it is._

## Foreign Function Interfaces (FFIs)

The Prose core client library is built in Rust. To communicate with its implementers, it exposes FFIs (for Foreign Function Interfaces) in various programming languages.

The supported programming languages are listed below:

* Swift

FFIs are built using the `prose_core_client_ffi` library contained in this project.

## Building & Testing

To build and test this library (using any of the provided examples), you can use `cargo run`. You will however need to pass the path to your local `libstrophe` library using `RUSTFLAGS`.

First off, you need to make sure that `libstrophe` is installed on your system:

```bash
brew install libstrophe
```

Then, you can run the `hello_bot` example as follows (this is for a [fish shell](https://fishshell.com/)):

```bash
set -x RUSTFLAGS "-L /opt/homebrew/Cellar/libstrophe/0.12.0/lib/"
set -x TEST_JID "your-name@your-domain.com"
set -x TEST_PASSWORD "<your_secret_password>"

cargo run --example hello_bot
```

Where `libstrophe` v0.12.0 was installed via Homebrew on macOS at the provided path.

_👉 Note that a convenience script is available under [scripts/run_test.sh](./scripts/run_test.sh). You can copy it to `scripts/run_local.sh` and inject your values, which will not get versioned to Git._

## License

Licensing information can be found in the [LICENSE.md](./LICENSE.md) document.

## :fire: Report A Vulnerability

If you find a vulnerability in any Prose system, you are more than welcome to report it directly to Prose Security by sending an encrypted email to [security@prose.org](mailto:security@prose.org). Do not report vulnerabilities in public GitHub issues, as they may be exploited by malicious people to target production systems running an unpatched version.

**:warning: You must encrypt your email using Prose Security GPG public key: [:key:57A5B260.pub.asc](https://files.prose.org/public/keys/gpg/57A5B260.pub.asc).**
