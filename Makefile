preflight:
	cargo fmt -- --check
	cargo build
	cargo build --package prose-sdk-js --target wasm32-unknown-unknown
	cargo build --package prose-sdk-ffi
	cargo build --package prose-core-client-cli --package xmpp-client
	cargo test --features test
	cargo test --package prose-core-integration-tests
	cargo test --package prose-store-integration-tests
	cargo xtask ci wasm
	cargo xtask ci wasm-store