test:
	@(export RUSTFLAGS="-L /opt/homebrew/Cellar/libstrophe/0.12.0/lib/"; cargo test --features "test-helpers" --package prose_core_client_ffi)