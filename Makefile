.PHONY: help install dev build test lint typecheck clean

help:
	@echo "Magrathea Writer - Developer Commands:"
	@echo "  make install    - Install frontend packages and check Rust dependencies"
	@echo "  make dev        - Launch desktop application in development mode"
	@echo "  make test       - Run all frontend and backend test suites"
	@echo "  make lint       - Run ESLint, Cargo clippy and Cargo fmt checks"
	@echo "  make typecheck  - Check Svelte and TypeScript static types"
	@echo "  make build      - Compile frontend bundle and verify Tauri build"

install:
	npm install
	cargo check --manifest-path src-tauri/Cargo.toml

dev:
	npm run tauri dev

test:
	npm test
	cargo test --manifest-path src-tauri/Cargo.toml

lint:
	npm run lint
	cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
	cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

typecheck:
	npm run typecheck

build:
	npm run build
	cargo check --manifest-path src-tauri/Cargo.toml
