.PHONY: all build test clean format lint

all: format lint build test

build:
	cargo build --workspace

test:
	cargo test --workspace
	cd contracts && forge test

format:
	cargo fmt --all
	cd contracts && forge fmt

lint:
	cargo clippy --workspace -- -D warnings

clean:
	cargo clean
	cd contracts && forge clean