build:
	RUSTFLAGS='-C link-arg=-s' cargo wasm

test:
	RUST_BACKTRACE=1 cargo test

schema:
	cargo schema

build-production:
	./bin/build

