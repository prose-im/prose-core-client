# prose-core-client

[![Test and Build](https://github.com/prose-im/prose-core-client/workflows/Test%20and%20Build/badge.svg?branch=master)](https://github.com/prose-im/prose-core-client/actions?query=workflow%3A%22Test+and+Build%22)

**Prose core XMPP client manager and protocols.**

_Tested at Rust version: `rustc 1.58.1 (db9d1b20b 2022-01-20)`_

## Architecture

The Prose core client uses the `libstrophe` library to provide with low-level XMPP protocol and connection management. It is built in C, and wrapped by a Rust library as to expose native Rust bindings.

It builds up upon this base layer and provides a common and identical interface to the XMPP world to Prose apps. Useful functions this library provides, for instance, are models and store management. This minimizes code reuse or code adaptations in different programming languages (eg. redefining a similar data model in Swift and Java at once).

The client library is organized into parts responsible for specific tasks, namely:

* Overall client manager: `client`
* Sending payloads: `broker/egress`
* Receiving payloads: `broker/ingress`
* Persistence: `store`
* Data types: `types`

## Foreign Function Interfaces (FFIs)

The Prose core client library is built in Rust. To communicate with its implementers, it exposes FFIs (for Foreign Function Interfaces) in various programming languages.

The supported programming languages are listed below:

* Swift

## Building & Testing

To build and test this library (using any of the provided examples), you can use `cargo run`. You will however need to pass the path to your local `libstrophe` library using `RUSTFLAGS`.

For example, you can run the `hello_bot` example as follows:

```bash
RUSTFLAGS="-L /opt/homebrew/Cellar/libstrophe/0.12.0/lib/" cargo run --example hello_bot
```

Where `libstrophe` v0.12.0 was installed via Homebrew on macOS at `/opt/homebrew/Cellar/libstrophe/0.12.0/`.

## License

Licensing information can be found in the [LICENSE.md](./LICENSE.md) document.

## :fire: Report A Vulnerability

If you find a vulnerability in any Prose system, you are more than welcome to report it directly to [@valeriansaliou](https://github.com/valeriansaliou) by sending an encrypted email to [valerian@valeriansaliou.name](mailto:valerian@valeriansaliou.name). Do not report vulnerabilities in public GitHub issues, as they may be exploited by malicious people to target production systems running an unpatched version.

**:warning: You must encrypt your email using [@valeriansaliou](https://github.com/valeriansaliou) GPG public key: [:key:valeriansaliou.gpg.pub.asc](https://valeriansaliou.name/files/keys/valeriansaliou.gpg.pub.asc).**
