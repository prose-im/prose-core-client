# prose-sdk-js

**Wasm/JS bindings for `prose-core-client`.**

To build this during development of [prose-web](https://github.com/prose-im/prose-web), in the [package.json](https://github.com/prose-im/prose-web/blob/master/package.json) replace the source of `@prose-im/prose-sdk-js` with `"file:/LOCAL/PATH/TO/prose-core-client/master/bindings/prose-sdk-js/pkg`" and run `cargo xtask wasm-pack build --dev` from the root folder of this repo to compile the crate.

Similarly, you can currently publish this library by running `cargo xtask wasm-pack publish`. Make sure however that `GITHUB_TOKEN` is available as an environment variable and contains a valid personal access token with the `repo` scope enabled.

_⚠️ Note that building this SDK currently requires the `nightly` Rust compiler._
