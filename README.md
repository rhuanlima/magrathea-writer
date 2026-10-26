# Magrathea Writer

🇬🇧 English · 🇧🇷 [Português](README.pt-BR.md)

*Your private workshop for building worlds.*

A **local-first, offline and encrypted** writing environment and literary "second brain" for fiction authors and worldbuilders. Your manuscript, world notes and timeline live in an **encrypted vault** on your machine: no other app can read your files. When you want to leave or share, **export** (open Markdown for Obsidian and friends, PDF, DOCX, EPUB).

> In Douglas Adams' *The Hitchhiker's Guide to the Galaxy*, Magrathea is the planet that builds custom planets, with hand-sculpted mountains and coastlines. This app is the workshop for yours.

**License:** [GNU AGPL v3](LICENSE) — free software; derivatives and network use must remain AGPL, crediting the original project. **No warranty of any kind: back up your vault and keep your recovery key safe** — losing both your password and recovery key means your work is unrecoverable by design.

**Repository:** https://github.com/rhuanlima/magrathea-writer

---

## Tech Stack

- **Core & Shell:** [Tauri 2](https://v2.tauri.app/) (Rust) — lightweight offline native window, memory-safe crypto and local vault storage.
- **Frontend UI:** [Svelte 5](https://svelte.dev/) + [TypeScript](https://www.typescriptlang.org/) + [Vite](https://vitejs.dev/) — ultra-responsive reactive UI without virtual DOM overhead.
- **Editor:** [CodeMirror 6](https://codemirror.net/) — distraction-free literary writing with dynamic focus and Markdown support.

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v20+ recommended) and `npm`
- [Rust](https://www.rust-lang.org/) toolchain (`rustc` and `cargo`)

### Installation & Development

```bash
# Install frontend and Rust dependencies
npm install
cargo check --manifest-path src-tauri/Cargo.toml

# Start development desktop app
npm run tauri dev
```

### Quality & Testing

```bash
# Run linting (ESLint, Cargo Clippy, Cargo fmt)
npm run lint
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# Typecheck Svelte and TypeScript
npm run typecheck

# Run test suites (Vitest & Cargo test)
npm test
cargo test --manifest-path src-tauri/Cargo.toml

# Production build
npm run build
npm run tauri build
```

---

## Contributing

Contributions require a DCO sign-off (`git commit -s`) by the human contributor. All development follows branch-based workflows with squash-merged Pull Requests.
