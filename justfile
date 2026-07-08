set dotenv-load

default:
    @just --list

setup:
    ./scripts/dev-setup.sh

dev:
    ./scripts/dev.sh

test:
    cargo test --workspace
    cd contracts && forge test
    cd delegation_circuit && nargo test
    cd frontend && npm run test

build-images:
    docker build -t otter-api .
    docker build -t otter-frontend ./frontend

smoke:
    ./scripts/smoke-test.sh
