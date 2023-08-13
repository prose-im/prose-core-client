# prose-core-client

[![Test and Build](https://github.com/prose-im/prose-core-client/actions/workflows/test.yml/badge.svg?branch=master)](https://github.com/prose-im/prose-core-client/actions/workflows/test.yml)

**Prose core XMPP client manager and protocols.**

Copyright 2022, Prose Foundation - Released under the [Mozilla Public License 2.0](./LICENSE.md).

_Tested at Rust version: `rustc 1.71.1 (eb26296b5 2023-08-03)`_

## SDKs

The Prose core client library is built in Rust. To communicate with its implementers, it exports SDKs in various programming languages.

The supported programming languages are listed below:

* Swift (FFIs)
* JavaScript (WebAssembly)

SDKs are built using the `bindings` packages contained in this project.

## License

Licensing information can be found in the [LICENSE.md](./LICENSE.md) document.

## :fire: Report A Vulnerability

If you find a vulnerability in any Prose system, you are more than welcome to report it directly to Prose Security by sending an encrypted email to [security@prose.org](mailto:security@prose.org). Do not report vulnerabilities in public GitHub issues, as they may be exploited by malicious people to target production systems running an unpatched version.

**:warning: You must encrypt your email using Prose Security GPG public key: [:key:57A5B260.pub.asc](https://files.prose.org/public/keys/gpg/57A5B260.pub.asc).**
