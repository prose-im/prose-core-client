# prose-sdk-js

**Wasm/JS bindings for `prose-core-client`.**

To build this during development of [prose-app-web](https://github.com/prose-im/prose-app-web), in the [package.json](https://github.com/prose-im/prose-app-web/blob/master/package.json) replace the source of `@prose-im/prose-sdk-js` with `"file:/LOCAL/PATH/TO/prose-core-client/master/bindings/prose-sdk-js/pkg`" and run `cargo xtask wasm-pack build --dev` from the root folder of this repo to compile the crate. You may also use the convenience `PROSE_CORE_CLIENT_PATH="../prose-core-client" npm run dev` command, passing the `PROSE_CORE_CLIENT_PATH` environment variable.

Similarly, you can currently publish this library by running `cargo xtask wasm-pack publish`. Make sure however that `GITHUB_TOKEN` is available as an environment variable and contains a valid personal access token with the `repo` scope enabled. Also, please provide a `NPM_TOKEN` environment variable for NPM publishing purposes.

_⚠️ Note that building this SDK currently requires the `nightly` Rust compiler._
