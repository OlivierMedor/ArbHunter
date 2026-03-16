.PHONY: all build test clean format lint forge-build forge-test

all: format lint build test

build:
	cargo build --workspace

test:
	cargo test --workspace
	docker compose run --rm forge forge test

forge-build:
	docker compose run --rm forge forge build

forge-test:
	docker compose run --rm forge forge test

format:
	cargo fmt --all
	docker compose run --rm forge forge fmt

lint:
	cargo clippy --workspace -- -D warnings

clean:
	cargo clean
	docker compose run --rm forge forge clean