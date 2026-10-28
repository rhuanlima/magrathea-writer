# Contributing to Magrathea Writer

Thank you for your interest in contributing to **Magrathea Writer**! Magrathea Writer is an offline, encrypted, distraction-free desktop workshop for novelists and worldbuilders, released under the **GNU Affero General Public License v3 (AGPLv3)**.

---

## 1. Principles & Hard Constraints

Before writing code or opening a pull request, please review our inviolable architectural principles:

1. **Absolute Offline Reality**: No network connections in runtime. Zero telemetry, zero external fonts/CDNs, no remote crash reports, no analytics.
2. **Encrypted Storage**: The author's literary work is sacred. All manuscript files and entities are encrypted at rest with **XChaCha20-Poly1305** authenticated envelope encryption and **Argon2id** key derivation. Never write plaintext temporary files.
3. **Copyleft Freedom (AGPLv3)**: All contributions are made under the AGPLv3.
4. **Developer Certificate of Origin (DCO)**: All human contributors must certify that they wrote or have the right to submit the code by adding a `Signed-off-by: Name <email>` trailer to commits.

---

## 2. Development Setup

Magrathea Writer is built with **Tauri v2**, **Svelte 5** (Runes), **CodeMirror 6**, and **Rust**.

### Prerequisites
- Node.js 20+ and `npm`
- Rust toolchain (stable)
- OS build dependencies for Tauri 2 (Xcode on macOS, build-essential on Linux, WebView2/MSVC on Windows)

### Commands
```bash
# Install dependencies
npm install
cargo check --manifest-path src-tauri/Cargo.toml

# Run in development mode
npm run tauri dev

# Run lint and code formatting checks
npm run lint
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# Run test suites
npm test
cargo test --manifest-path src-tauri/Cargo.toml

# Build release package
npm run tauri build
```

---

## 3. Pull Request Guidelines

1. **One topic per PR**: Focus on a single feature, bugfix, or improvement.
2. **Conventional Commits**: Use descriptive messages such as `feat(editor): add focus mode` or `fix(vault): prevent race condition in autosave`.
3. **Tests Required**: Every feature or bugfix must include automated unit/integration tests (`vitest` or `cargo test`).
4. **CI Gate**: PRs are merged via squash only after all GitHub Actions checks (lint, tests, security audits) pass.
