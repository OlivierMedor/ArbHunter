import os

cargo_toml = """[workspace]
resolver = "2"
members = [
  "bin/arb_daemon",
  "crates/arb_types",
  "crates/arb_config",
  "crates/arb_metrics",
  "crates/arb_providers",
  "crates/arb_ingest",
  "crates/arb_state",
  "crates/arb_filter",
  "crates/arb_route",
  "crates/arb_sim",
  "crates/arb_execute",
  "crates/arb_storage",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
publish = false

[workspace.dependencies]
# Dependencies will be added in Phase 2+
"""

docker_compose = """version: "3.8"

services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: ${DB_USER:-arb}
      POSTGRES_PASSWORD: ${DB_PASS:-arb}
      POSTGRES_DB: ${DB_NAME:-arb_metrics}
    ports:
      - "5432:5432"
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U arb"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  pgdata:
"""

makefile = """.PHONY: all build test clean format lint

all: format lint build test

build:
\tcargo build --workspace

test:
\tcargo test --workspace
\tcd contracts && forge test

format:
\tcargo fmt --all
\tcd contracts && forge fmt

lint:
\tcargo clippy --workspace -- -D warnings

clean:
\tcargo clean
\tcd contracts && forge clean
"""

env_example = """# ArbHunter Environment Variables

# RPC Endpoints
QUICKNODE_HTTP_URL=http://localhost:8545
ALCHEMY_HTTP_URL=
TENDERLY_RPC_URL=

# Database
DB_USER=arb
DB_PASS=arb
DB_NAME=arb_metrics
DB_HOST=localhost
DB_PORT=5432

# Logging & Metrics
RUST_LOG=info
METRICS_PORT=9090
"""

foundry_toml = """[profile.default]
src = "src"
out = "out"
libs = ["lib"]

[fmt]
line_length = 120
tab_width = 4
bracket_spacing = true
"""

files = {
    "Cargo.toml": cargo_toml,
    "docker-compose.yml": docker_compose,
    "Makefile": makefile,
    ".env.example": env_example,
    "contracts/foundry.toml": foundry_toml
}

for filename, content in files.items():
    with open(filename, 'wb') as f:
        f.write(content.replace('\r\n', '\n').encode('utf-8'))

print("Files strictly overwritten with LF.")
