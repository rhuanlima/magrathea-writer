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

## Key Features

- 🔒 **Encrypted World Vaults**: Full local envelope encryption (XChaCha20-Poly1305 + Argon2id). Pure zero-knowledge, zero-telemetry, 100% offline.
- ✍️ **Distraction-Free Literary Editor**: Dynamic line focus, Zen mode, pure white clean interface, status and words HUD, custom typography (EB Garamond).
- 🌳 **Manuscript Hierarchy**: Organize by Parts, Chapters and Scenes, with direct narrative text composition support on every level.
- ⚡ **Global Command Palette (`Cmd+K` / `Ctrl+K`)**: Instant search and navigation across manuscript scenes and world dossiers, with full keyboard operation.
- 🧠 **Literary Second Brain & Worldbuilding**: Character arcs, places, items, lore dossiers, bidirectional `[[wikilinks]]`, and in-memory backlink index.
- ⏳ **Timeline & Chronology**: Event sequencing, chronological vs. narrative ordering, analepsis/prolepsis tracking and chapter pacing analysis.
- 🕸️ **Narrative Graph & JSON Canvas**: Visual relationship graph with force simulation, orphan detection, and infinite mind-map canvas compatible with the open JSON Canvas standard.
- 📖 **Direct Native Export**: Export seamlessly to Microsoft Word (.docx), EPUB3 e-book, printable PDF/HTML, Obsidian Markdown vault (.md), and plain text with dossiers.

---

## Contributing

Please review [CONTRIBUTING.md](CONTRIBUTING.md) and our [Code of Conduct](CODE_OF_CONDUCT.md). All human contributions require a DCO sign-off (`git commit -s`).

For third-party licenses and attribution notice, see [NOTICE](NOTICE). For brand policy, see [TRADEMARK.md](TRADEMARK.md).

---

## Closed Beta for Authors

Are you testing Magrathea Writer as an invited author? Please read our [Closed Beta Welcome Guide](BETA_GUIDE.md).
